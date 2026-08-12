//! From-text observation driver (issue #72): feed natural text through the
//! teacher and record the SAME v3 observation records the autoregressive
//! `observe` path produces, so the sealed D3 natural partition corpus
//! (Simple English Wikipedia, `.uor-models/corpora/simple-wiki-20231101`)
//! becomes a real observation corpus.
//!
//! Record semantics are the generation path's, teacher-forced:
//!
//! - per article, the text is tokenized BOS-prefixed and the oracle steps
//!   over the stream; at each position the v4 88-byte record for
//!   (8-token context window → next text token) is emitted through the
//!   shared [`compiler::encode_v4_record`] /
//!   [`compiler::softmax_top8_sample_with_stats`] /
//!   [`compiler::byte_anchors`] helpers, with aligned probability metadata
//!   preserving the full-distribution target likelihood and entropy. The
//!   sampled token is discarded; the record's `next` is the actual next text
//!   token;
//! - sharding is the generation path's scheme: [`observe::sample_id`] over
//!   the context window → [`observe::shard_of`] — content-addressed, so
//!   shard bytes are independent of article completion order (T-invariance);
//! - `story` is the article ordinal (u32, dense, in jsonl order); the
//!   story → article mapping is written to `stories.jsonl` (one JSON object
//!   per line: story, id, url, title, partition).
//!
//! Partition semantics (D3): the split rule of the corpus manifest —
//! held-out = `blake3(article id as utf-8)[0] % 5 == 0` — is applied AT
//! WRITE TIME by [`partition_of`]: every record is tagged with its
//! article's partition, each shard's manifest entry lists the
//! construction/held-out record counts, and the observation manifest
//! records the rule itself, so downstream consumers can split a merged
//! corpus exactly (`stories.jsonl` carries the per-story partition).
//!
//! Resume contract: per-article checkpointing. `committed.bin` is the
//! authoritative checkpoint (the 25-byte corpus-meta header — n, stories,
//! rng, done — plus the input κ and per-shard committed byte lengths and
//! partition counts, atomically renamed); `state.bin` mirrors the header
//! in the exact 25-byte corpus-meta layout for readers shared with the
//! generation path. A rerun skips completed shards (manifest) and
//! completed articles (the dense ordinal prefix). Byte-level resume within
//! an article is not needed: an interrupted article is restarted, its
//! records are content-stable, and incomplete shards/story lines are
//! trimmed back to the committed checkpoint on open, so a resumed run
//! converges to the exact shard κ of a single-pass run.

use super::compiler;
use crate::observation::{
    self as observe, ObservationShardWriter, PartitionCounts, RECORD_SIZE, RecordPartition,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use uor_r4_core::transformerless::hf_bpe::TokenizerKind;
use uor_r4_model_source::SourceUnavailable;
use uor_r4_model_source::TeacherOracle;
use uor_r4_model_source::progress::Progress;
use uor_r4_model_source::{BatchedTeacher, State};

/// Authoritative per-article checkpoint file name within an observation
/// directory.
pub const COMMITTED_FILE: &str = "committed.bin";

/// Story → article mapping file name within an observation directory.
pub const STORIES_FILE: &str = "stories.jsonl";

/// The document-level partition rule, recorded in the observation manifest
/// verbatim from the sealed corpus manifest (`manifest.json` split_rule).
pub const PARTITION_RULE: &str =
    "held-out = blake3(article id as utf-8)[0] % 5 == 0; remainder is construction";

/// rng seed of the observation stream, identical to the corpus and
/// autoregressive observation streams.
const RNG_SEED: u64 = 0x5EED;

/// Checkpoint header width: the corpus-meta layout (n u64 | stories u64 |
/// rng u64 | done u8), mirrored verbatim into `state.bin`.
const HEADER_SIZE: usize = 25;

/// Per-shard checkpoint row width: committed byte length u64 |
/// construction records u64 | held-out records u64.
const SHARD_ROW_SIZE: usize = 24;

/// Input-pin width: blake3 digest of the articles file.
const INPUT_KAPPA_SIZE: usize = 32;

/// Document-level partition of one article, keyed by its article id:
/// held-out when `blake3(id as utf-8)[0] % 5 == 0` (the D3 split rule).
pub fn partition_of(article_id: &str) -> RecordPartition {
    if blake3::hash(article_id.as_bytes()).as_bytes()[0].is_multiple_of(5) {
        RecordPartition::HeldOut
    } else {
        RecordPartition::Construction
    }
}

/// One article of the sealed text corpus (one JSON object per line).
#[derive(Debug, Deserialize)]
struct Article {
    id: String,
    url: String,
    title: String,
    text: String,
}

/// One line of `stories.jsonl`: the story ordinal → article mapping with
/// the article's partition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryEntry {
    pub story: u32,
    pub id: String,
    pub url: String,
    pub title: String,
    pub partition: RecordPartition,
}

/// The loaded story → article mapping of an observation directory.
#[derive(Debug, Clone)]
pub struct StoryIndex {
    entries: Vec<StoryEntry>,
}

impl StoryIndex {
    /// Load `stories.jsonl`, validating dense ordinals (line i must map
    /// story i). Returns `Ok(None)` when the file does not exist.
    pub fn load(path: &Path) -> Result<Option<Self>, SourceUnavailable> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(SourceUnavailable::new(format!(
                    "{}: {error}",
                    path.display()
                )));
            }
        };
        let mut entries = Vec::new();
        for (index, line) in bytes.split(|&byte| byte == b'\n').enumerate() {
            if line.is_empty() {
                continue;
            }
            let entry: StoryEntry = serde_json::from_slice(line).map_err(|error| {
                SourceUnavailable::new(format!(
                    "{} line {}: invalid story entry: {error}",
                    path.display(),
                    index + 1
                ))
            })?;
            if entry.story as usize != entries.len() {
                return Err(SourceUnavailable::new(format!(
                    "{} line {}: story ordinals are not dense (got {}, expected {})",
                    path.display(),
                    index + 1,
                    entry.story,
                    entries.len()
                )));
            }
            entries.push(entry);
        }
        Ok(Some(Self { entries }))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The mapping entry of one story ordinal.
    pub fn get(&self, story: u32) -> Option<&StoryEntry> {
        self.entries.get(story as usize)
    }

    /// The document-level partition of one story ordinal.
    pub fn partition_of(&self, story: u32) -> Option<RecordPartition> {
        self.get(story).map(|entry| entry.partition)
    }

    /// (construction, held-out) article counts across the mapping.
    pub fn partition_counts(&self) -> (u64, u64) {
        let mut counts = (0u64, 0u64);
        for entry in &self.entries {
            match entry.partition {
                RecordPartition::Construction => counts.0 += 1,
                RecordPartition::HeldOut => counts.1 += 1,
            }
        }
        counts
    }
}

/// Per-shard committed state: byte length of the shard file covered by the
/// checkpoint plus the partition counts of the records in it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ShardCheckpoint {
    bytes: u64,
    partitions: PartitionCounts,
}

/// The authoritative checkpoint: the corpus-meta header (n, stories, rng,
/// done), the input κ pinning the articles file across resumes, and the
/// per-shard committed state.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Checkpoint {
    n: u64,
    stories: u64,
    rng: u64,
    done: bool,
    input_kappa: [u8; INPUT_KAPPA_SIZE],
    shards: Vec<ShardCheckpoint>,
}

impl Checkpoint {
    fn fresh(shard_count: u32, input_kappa: [u8; INPUT_KAPPA_SIZE]) -> Self {
        Self {
            n: 0,
            stories: 0,
            rng: RNG_SEED,
            done: false,
            input_kappa,
            shards: vec![ShardCheckpoint::default(); shard_count as usize],
        }
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes =
            Vec::with_capacity(HEADER_SIZE + INPUT_KAPPA_SIZE + self.shards.len() * SHARD_ROW_SIZE);
        bytes.extend_from_slice(&self.header());
        bytes.extend_from_slice(&self.input_kappa);
        for shard in &self.shards {
            bytes.extend_from_slice(&shard.bytes.to_le_bytes());
            bytes.extend_from_slice(&shard.partitions.construction.to_le_bytes());
            bytes.extend_from_slice(&shard.partitions.held_out.to_le_bytes());
        }
        bytes
    }

    /// The 25-byte corpus-meta header: n u64 | stories u64 | rng u64 |
    /// done u8 — the `state.bin` mirror layout.
    fn header(&self) -> [u8; HEADER_SIZE] {
        let mut header = [0u8; HEADER_SIZE];
        header[0..8].copy_from_slice(&self.n.to_le_bytes());
        header[8..16].copy_from_slice(&self.stories.to_le_bytes());
        header[16..24].copy_from_slice(&self.rng.to_le_bytes());
        header[24] = u8::from(self.done);
        header
    }

    fn decode(bytes: &[u8], shard_count: u32) -> Result<Self, SourceUnavailable> {
        let expected = HEADER_SIZE + INPUT_KAPPA_SIZE + shard_count as usize * SHARD_ROW_SIZE;
        if bytes.len() != expected {
            return Err(SourceUnavailable::new(format!(
                "committed checkpoint has {} bytes, expected {expected}",
                bytes.len()
            )));
        }
        let at = |offset: usize| {
            u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8-byte slice"))
        };
        let mut shards = Vec::with_capacity(shard_count as usize);
        let mut offset = HEADER_SIZE + INPUT_KAPPA_SIZE;
        for _ in 0..shard_count {
            let shard = ShardCheckpoint {
                bytes: at(offset),
                partitions: PartitionCounts {
                    construction: at(offset + 8),
                    held_out: at(offset + 16),
                },
            };
            if !shard.bytes.is_multiple_of(RECORD_SIZE as u64) {
                return Err(SourceUnavailable::new(
                    "committed checkpoint has a torn shard length",
                ));
            }
            shards.push(shard);
            offset += SHARD_ROW_SIZE;
        }
        let checkpoint = Self {
            n: at(0),
            stories: at(8),
            rng: at(16),
            done: bytes[24] != 0,
            input_kappa: bytes[HEADER_SIZE..HEADER_SIZE + INPUT_KAPPA_SIZE]
                .try_into()
                .expect("32-byte slice"),
            shards,
        };
        let committed_records: u64 = checkpoint
            .shards
            .iter()
            .map(|shard| shard.bytes / RECORD_SIZE as u64)
            .sum();
        if checkpoint.n != committed_records {
            return Err(SourceUnavailable::new(format!(
                "committed checkpoint records {} do not match the shard lengths {committed_records}",
                checkpoint.n
            )));
        }
        Ok(checkpoint)
    }
}

fn input_kappa(path: &Path) -> Result<[u8; INPUT_KAPPA_SIZE], SourceUnavailable> {
    let bytes = fs::read(path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

fn read_checkpoint(dir: &Path, shard_count: u32) -> Result<Option<Checkpoint>, SourceUnavailable> {
    let path = dir.join(COMMITTED_FILE);
    match fs::read(&path) {
        Ok(bytes) => Checkpoint::decode(&bytes, shard_count).map(Some),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(SourceUnavailable::new(format!(
            "{}: {error}",
            path.display()
        ))),
    }
}

/// Persist the checkpoint: `committed.bin` atomically (write-then-rename),
/// then the `state.bin` mirror in the 25-byte corpus-meta layout.
fn write_checkpoint(dir: &Path, checkpoint: &Checkpoint) -> Result<(), SourceUnavailable> {
    let tmp = dir.join(".committed.bin.tmp");
    fs::write(&tmp, checkpoint.encode())
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", tmp.display())))?;
    fs::rename(&tmp, dir.join(COMMITTED_FILE))
        .map_err(|error| SourceUnavailable::new(format!("committed checkpoint rename: {error}")))?;
    let state_path = dir.join(observe::STATE_FILE);
    fs::write(&state_path, checkpoint.header())
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", state_path.display())))?;
    Ok(())
}

/// Trim one incomplete shard file back to its committed length, so a
/// restarted article never duplicates its records. A file longer than the
/// checkpoint holds the content-stable tail of an interrupted article and
/// is truncated; a file shorter than the checkpoint means data loss.
fn reconcile_shard(
    dir: &Path,
    shard_bits: u8,
    shard: u32,
    committed: u64,
) -> Result<(), SourceUnavailable> {
    let path = dir.join(observe::shard_file_name(shard_bits, shard));
    let length = match fs::metadata(&path) {
        Ok(metadata) => metadata.len(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if committed == 0 {
                return Ok(());
            }
            return Err(SourceUnavailable::new(format!(
                "{} is missing but the checkpoint commits {committed} bytes; delete the observation directory and rerun",
                path.display()
            )));
        }
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{}: {error}",
                path.display()
            )));
        }
    };
    if length % RECORD_SIZE as u64 != 0 {
        return Err(SourceUnavailable::new(format!(
            "shard file {} has a torn record ({length} bytes); delete it and rerun",
            path.display()
        )));
    }
    if length < committed {
        return Err(SourceUnavailable::new(format!(
            "{} is shorter ({length} bytes) than the committed checkpoint ({committed} bytes); delete the observation directory and rerun",
            path.display()
        )));
    }
    if length > committed {
        let file = fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
        file.set_len(committed)
            .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
    }
    Ok(())
}

/// Trim `stories.jsonl` back to the committed story count (crash window:
/// a story line appended just before the checkpoint rename failed).
fn reconcile_stories(path: &Path, stories: u64) -> Result<(), SourceUnavailable> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if stories == 0 {
                return Ok(());
            }
            return Err(SourceUnavailable::new(format!(
                "{} is missing but the checkpoint commits {stories} stories; delete the observation directory and rerun",
                path.display()
            )));
        }
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "{}: {error}",
                path.display()
            )));
        }
    };
    let mut lines: Vec<&[u8]> = bytes.split(|&byte| byte == b'\n').collect();
    if lines.last() == Some(&b"".as_slice()) {
        lines.pop();
    }
    if (lines.len() as u64) < stories {
        return Err(SourceUnavailable::new(format!(
            "{} has {} story lines but the checkpoint commits {stories}; delete the observation directory and rerun",
            path.display(),
            lines.len()
        )));
    }
    if lines.len() as u64 == stories {
        return Ok(());
    }
    let mut trimmed = Vec::new();
    for line in &lines[..stories as usize] {
        trimmed.extend_from_slice(line);
        trimmed.push(b'\n');
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &trimmed)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", tmp.display())))?;
    fs::rename(&tmp, path).map_err(|error| {
        SourceUnavailable::new(format!("{}: story mapping rename: {error}", path.display()))
    })?;
    Ok(())
}

/// Append one story mapping line to `stories.jsonl`.
fn append_story(path: &Path, entry: &StoryEntry) -> Result<(), SourceUnavailable> {
    let mut line = serde_json::to_vec(entry)?;
    line.push(b'\n');
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
    file.write_all(&line)
        .and_then(|()| file.flush())
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", path.display())))?;
    Ok(())
}

/// Outcome of one [`observe_text_corpus`] invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ObservationReport {
    /// Articles in the input file.
    pub articles_total: u64,
    /// Articles committed so far (= the next article ordinal to process).
    pub articles_completed: u64,
    /// Articles truncated at the teacher sequence length during this
    /// invocation.
    pub articles_truncated: u64,
    /// Characters replaced by the lossy tokenizer fallback during this
    /// invocation (unencodable in the teacher vocab; substituted with a
    /// space — deterministic, see the legacy `Tokenizer::encode_lossy`;
    /// always zero on the byte-level BPE path, which encodes every input).
    pub characters_replaced: u64,
    /// Records committed so far (all invocations).
    pub records: u64,
    /// Records written during this invocation.
    pub written: u64,
    /// Committed construction records (all shards).
    pub construction_records: u64,
    /// Committed held-out records (all shards).
    pub held_out_records: u64,
    /// Construction articles in the committed story mapping.
    pub construction_articles: u64,
    /// Held-out articles in the committed story mapping.
    pub held_out_articles: u64,
    /// Shards κ-pinned in the manifest.
    pub shards_completed: u32,
    /// Shards in the configured fan-out.
    pub shard_count: u32,
    /// κ of the merged shard bytes, once the corpus is complete.
    pub merged_kappa: Option<String>,
    /// Teacher-forced cross-entropy of the observed message stream, in
    /// bits/token. This is computed from the probability sidecar rather than
    /// from top-k-renormalized evidence.
    pub teacher_bits_per_token: Option<f64>,
    /// Path of the story → article mapping file.
    pub stories_file: PathBuf,
    /// Whether every article is committed and every shard is κ-pinned.
    pub done: bool,
}

fn build_report(
    out_dir: &Path,
    checkpoint: &Checkpoint,
    writer: &ObservationShardWriter,
    articles_total: u64,
    articles_truncated: u64,
    characters_replaced: u64,
    written: u64,
) -> Result<ObservationReport, SourceUnavailable> {
    let stories_path = out_dir.join(STORIES_FILE);
    let (construction_articles, held_out_articles) = match StoryIndex::load(&stories_path)? {
        Some(index) => index.partition_counts(),
        None => (0, 0),
    };
    let (construction_records, held_out_records) =
        checkpoint
            .shards
            .iter()
            .fold((0u64, 0u64), |(construction, held_out), shard| {
                (
                    construction + shard.partitions.construction,
                    held_out + shard.partitions.held_out,
                )
            });
    let merged_kappa = if checkpoint.done {
        let merged = observe::merge_shards(out_dir)?;
        Some(format!("blake3:{}", blake3::hash(&merged).to_hex()))
    } else {
        None
    };
    let teacher_bits_per_token = if checkpoint.done {
        observe::merge_probability_metadata(out_dir)
            .ok()
            .and_then(|metadata| observe::message_bits_per_token(&metadata))
    } else {
        None
    };
    Ok(ObservationReport {
        articles_total,
        articles_completed: checkpoint.stories,
        articles_truncated,
        characters_replaced,
        records: checkpoint.n,
        written,
        construction_records,
        held_out_records,
        construction_articles,
        held_out_articles,
        shards_completed: writer.manifest().completed.len() as u32,
        shard_count: writer.manifest().shard_count(),
        merged_kappa,
        teacher_bits_per_token,
        stories_file: stories_path,
        done: checkpoint.done,
    })
}

/// Run the from-text observation pass over `articles_path` (one JSON
/// object per line: id, url, title, text), spilling v4 records into
/// content-addressed shards under `out_dir`.
///
/// The teacher stream is teacher-forced: per article the BOS-prefixed
/// token stream is stepped through the oracle and each position records
/// (context window → actual next token). Positions are capped at the
/// oracle's sequence length (longer articles are truncated). The pass
/// checkpoints per article and stops when `budget_s` elapses; rerunning
/// resumes from the checkpoint. With `resume` set, an existing observation
/// directory continues from its checkpoint; without it, a non-empty
/// directory is an error.
#[allow(clippy::too_many_arguments)] // mirrors the observe_sharded driver signature
/// The per-article product of the teacher-forced pass: the ordered records
/// (each with its shard and probability sidecar), the story-mapping entry, and
/// the truncation/replacement counters. Pure — it neither writes shards nor
/// touches the checkpoint — so it can run on a worker thread with its own
/// oracle while a single-threaded collector commits the results in article
/// order. Because the writer appends in call order, committing produced
/// articles in ascending ordinal (and each article's positions in order)
/// reproduces the exact shard bytes of a single-pass run.
struct ArticleProduced {
    ordinal: u64,
    records: Vec<(u32, [u8; RECORD_SIZE], observe::ProbabilityMetadata)>,
    story_entry: StoryEntry,
    truncated: bool,
    replaced: u64,
}

/// One encoded teacher-forced position: its record, the shard it routes to, and
/// the aligned probability sidecar.
struct EncodedPosition {
    shard: u32,
    record: [u8; RECORD_SIZE],
    metadata: observe::ProbabilityMetadata,
}

/// Encode the observation record at one teacher-forced position from that
/// position's `logits`. Pushes `token` onto `window` (trimmed to the context
/// width), routes by the content-addressed sample id, and returns the record
/// plus the advanced story byte offset. Shared by the serial and batched
/// drivers so both emit identical records for identical logits. The sampled
/// token is discarded — every recorded field is deterministic from
/// `(logits, token, next)`.
#[allow(clippy::too_many_arguments)]
fn encode_position(
    logits: &mut [f32],
    story: u32,
    pos: usize,
    token: u32,
    next: u32,
    window: &mut Vec<u32>,
    story_byte_offset: u32,
    token_byte_lengths: Option<&[u32]>,
    shard_bits: u8,
    rng: &mut u64,
) -> (EncodedPosition, u32) {
    let (_sampled, top_tokens, top_weights, sampled_stats) =
        compiler::softmax_top8_sample_with_stats(logits, rng);
    let stats = compiler::TokenProbabilityStats::from_normalized(
        logits,
        next as usize,
        &top_tokens,
        sampled_stats.top8_mass,
    );
    window.push(token);
    if window.len() > compiler::WINDOW {
        window.remove(0);
    }
    let id = observe::sample_id(window);
    let shard = observe::shard_of(&id, shard_bits);
    let span_start = pos as u32;
    let span_end = span_start.saturating_add(1);
    let (byte_start, byte_end) =
        compiler::byte_anchors(token_byte_lengths, story_byte_offset, next as usize);
    let record = compiler::encode_v4_record(
        story,
        next,
        &top_tokens,
        &top_weights,
        (span_start, span_end),
        (byte_start, byte_end),
    );
    let metadata = observe::ProbabilityMetadata {
        target_logprob_nats: stats.target_logprob_nats,
        entropy_bits: stats.entropy_bits,
        top8_mass: stats.top8_mass,
        target_rank: stats.target_rank,
    };
    let advanced = if token_byte_lengths.is_some() {
        byte_end
    } else {
        story_byte_offset
    };
    (
        EncodedPosition {
            shard,
            record,
            metadata,
        },
        advanced,
    )
}

/// Teacher-force one article and return its record stream. The sampled token
/// is discarded (see the module docs): every recorded field is a deterministic
/// function of `(article tokens, model)`, so this is independent of worker,
/// order, and the throwaway rng seeded below.
#[allow(clippy::too_many_arguments)]
fn produce_article_records(
    oracle: &mut dyn TeacherOracle,
    ordinal: u64,
    article: &Article,
    seq_len: usize,
    tokenizer: &TokenizerKind,
    token_byte_lengths: Option<&[u32]>,
    shard_bits: u8,
    rng: &mut u64,
) -> Result<ArticleProduced, SourceUnavailable> {
    let story = u32::try_from(ordinal)
        .map_err(|_| SourceUnavailable::new("article ordinal exceeds the u32 story field"))?;
    let partition = partition_of(&article.id);
    let (tokens, replaced) = tokenizer.encode_lossy(&article.text);
    let positions = tokens.len().saturating_sub(1).min(seq_len);
    let truncated = positions < tokens.len().saturating_sub(1);

    let mut logits = vec![0f32; oracle.vocab()];
    let mut window: Vec<u32> = Vec::with_capacity(compiler::WINDOW);
    let mut records = Vec::with_capacity(positions);
    let mut story_byte_offset = 0u32;
    oracle.reset();
    for pos in 0..positions {
        let token = tokens[pos];
        oracle.step(token as usize, pos, &mut logits);
        let next = tokens[pos + 1];
        let (encoded, advanced) = encode_position(
            &mut logits,
            story,
            pos,
            token,
            next,
            &mut window,
            story_byte_offset,
            token_byte_lengths,
            shard_bits,
            rng,
        );
        records.push((encoded.shard, encoded.record, encoded.metadata));
        story_byte_offset = advanced;
    }
    Ok(ArticleProduced {
        ordinal,
        records,
        story_entry: StoryEntry {
            story,
            id: article.id.clone(),
            url: article.url.clone(),
            title: article.title.clone(),
            partition,
        },
        truncated,
        replaced,
    })
}

/// Commit one produced article's records in order: append them to their
/// shards, roll the per-shard partition counts, write the story-mapping line,
/// and advance + persist the checkpoint. The single point that mutates the
/// writer and checkpoint, so applying produced articles here in ascending
/// ordinal reproduces a single-pass run's shard bytes exactly.
#[allow(clippy::too_many_arguments)]
fn commit_article(
    writer: &mut ObservationShardWriter,
    checkpoint: &mut Checkpoint,
    out_dir: &Path,
    stories_path: &Path,
    produced: &ArticleProduced,
    articles_total: u64,
    written: &mut u64,
    truncated: &mut u64,
    replaced: &mut u64,
) -> Result<(), SourceUnavailable> {
    if produced.truncated {
        *truncated += 1;
    }
    *replaced += produced.replaced;
    checkpoint.n += produced.records.len() as u64;
    // Per-article checkpoint: shard bytes first (flush), then the story
    // mapping line, then the atomic committed checkpoint and its state.bin
    // mirror.
    for (shard, record, probability) in &produced.records {
        if writer.write_record_with_probability_in_partition(
            record,
            *probability,
            *shard,
            produced.story_entry.partition,
        )? {
            *written += 1;
            checkpoint.shards[*shard as usize].bytes += RECORD_SIZE as u64;
        }
    }
    writer.flush()?;
    for (slot, shard_checkpoint) in checkpoint.shards.iter_mut().enumerate() {
        shard_checkpoint.partitions = writer.partition_counts(slot as u32).unwrap_or_default();
    }
    append_story(stories_path, &produced.story_entry)?;
    let next_ordinal = produced.ordinal + 1;
    checkpoint.stories = next_ordinal;
    checkpoint.done = next_ordinal == articles_total;
    write_checkpoint(out_dir, checkpoint)?;
    Ok(())
}

/// Outcome of opening an observation directory: either the corpus is already
/// complete (its report), or a fresh/resumed session ready to receive records.
enum Prepared {
    Done(ObservationReport),
    Ready {
        /// Boxed: the manifest-carrying writer dominates the variant size
        /// (clippy::large_enum_variant since the #600 geometry record).
        writer: Box<ObservationShardWriter>,
        checkpoint: Checkpoint,
        articles_total: u64,
        stories_path: PathBuf,
    },
}

/// Open an observation directory: validate/resume the checkpoint, reconcile any
/// interrupted on-disk shard/story tails, and count the article stream. Shared
/// by the serial and batched drivers — everything up to the per-article loop is
/// identical regardless of how records are produced.
fn prepare_text_observation(
    out_dir: &Path,
    articles_path: &Path,
    shard_bits: u8,
    resume: bool,
) -> Result<Prepared, SourceUnavailable> {
    let kappa = input_kappa(articles_path)?;
    let mut writer = ObservationShardWriter::open(out_dir, shard_bits)?;
    let shard_count = writer.manifest().shard_count();
    let stories_path = out_dir.join(STORIES_FILE);
    let has_prior_state = out_dir.join(COMMITTED_FILE).exists()
        || out_dir.join(observe::STATE_FILE).exists()
        || stories_path.exists()
        || !writer.manifest().completed.is_empty()
        || (0..shard_count).any(|shard| {
            out_dir
                .join(observe::shard_file_name(shard_bits, shard))
                .exists()
        });
    if !resume && has_prior_state {
        return Err(SourceUnavailable::new(format!(
            "{} already contains an observation corpus; pass resume to continue it",
            out_dir.display()
        )));
    }
    writer.set_partition_rule(PARTITION_RULE)?;
    // The PROV link from produced artifacts back to the sealed corpus
    // (issue #72): the input κ is the corpus CID of the D3 manifest.
    writer.set_input_cid(&format!("blake3:{}", blake3::Hash::from(kappa).to_hex()))?;

    let checkpoint = match read_checkpoint(out_dir, shard_count)? {
        Some(checkpoint) => {
            if checkpoint.input_kappa != kappa {
                return Err(SourceUnavailable::new(format!(
                    "{} does not match the observation checkpoint's input; pass the same articles file or a fresh output directory",
                    articles_path.display()
                )));
            }
            checkpoint
        }
        None => Checkpoint::fresh(shard_count, kappa),
    };
    if !checkpoint.done && !writer.manifest().completed.is_empty() {
        return Err(SourceUnavailable::new(format!(
            "{} has finalized shards but an unfinished checkpoint; delete the observation directory and rerun",
            out_dir.display()
        )));
    }
    // Reconcile on-disk bytes to the committed checkpoint before writing:
    // interrupted articles leave content-stable tails that are trimmed, so
    // the restarted article's records are appended exactly once.
    for shard in 0..shard_count {
        if writer.is_complete(shard) {
            continue;
        }
        reconcile_shard(
            out_dir,
            shard_bits,
            shard,
            checkpoint.shards[shard as usize].bytes,
        )?;
        observe::reconcile_probability_shard(
            out_dir,
            shard_bits,
            shard,
            checkpoint.shards[shard as usize].bytes,
        )?;
    }
    reconcile_stories(&stories_path, checkpoint.stories)?;
    let counts: Vec<PartitionCounts> = checkpoint
        .shards
        .iter()
        .map(|shard| shard.partitions)
        .collect();
    writer.restore_partition_counts(&counts)?;

    // The article stream is processed in jsonl order; count it up front
    // for progress and the report.
    let articles_total = {
        let file = fs::File::open(articles_path).map_err(|error| {
            SourceUnavailable::new(format!("{}: {error}", articles_path.display()))
        })?;
        let mut lines = BufReader::new(file).lines();
        let mut total = 0u64;
        for line in &mut lines {
            line.map_err(|error| {
                SourceUnavailable::new(format!("{}: {error}", articles_path.display()))
            })?;
            total += 1;
        }
        total
    };

    if checkpoint.done {
        // A crash between the done checkpoint and finalization can leave
        // shards unpinned; finalize (idempotent) and stop without touching
        // completed shard files.
        writer.finalize_all()?;
        println!(
            "text observation corpus already complete: {} records",
            checkpoint.n
        );
        let report = build_report(out_dir, &checkpoint, &writer, articles_total, 0, 0, 0)?;
        return Ok(Prepared::Done(report));
    }

    Ok(Prepared::Ready {
        writer: Box::new(writer),
        checkpoint,
        articles_total,
        stories_path,
    })
}

/// Observe a text corpus with a pool of `oracles` teacher instances. Articles
/// are teacher-forced and mutually independent (the sampled token is discarded;
/// every recorded field is a deterministic function of the article tokens and
/// the model), so a batch of up to `oracles.len()` articles is produced in
/// parallel — one worker thread per oracle — and then committed on this thread
/// in ascending article ordinal. Because the shard writer appends in call
/// order, the committed shard bytes are identical to a single-oracle pass.
///
/// A single-oracle pool (`oracles.len() == 1`) runs entirely on this thread and
/// threads `checkpoint.rng` exactly as the original in-line loop did, so it is
/// byte-for-byte unchanged including the committed checkpoint.
#[allow(clippy::too_many_arguments)]
pub fn observe_text_corpus(
    oracles: &mut [Box<dyn TeacherOracle + Send>],
    budget_s: u64,
    tokenizer: &TokenizerKind,
    token_byte_lengths: Option<&[u32]>,
    articles_path: &Path,
    out_dir: &Path,
    shard_bits: u8,
    resume: bool,
) -> Result<ObservationReport, SourceUnavailable> {
    assert!(
        !oracles.is_empty(),
        "observe_text_corpus needs at least one oracle"
    );
    let (mut writer, mut checkpoint, articles_total, stories_path) =
        match prepare_text_observation(out_dir, articles_path, shard_bits, resume)? {
            Prepared::Done(report) => return Ok(report),
            Prepared::Ready {
                writer,
                checkpoint,
                articles_total,
                stories_path,
            } => (writer, checkpoint, articles_total, stories_path),
        };

    // #600: record the typed geometry-projection record the teacher
    // oracle declares (the source→compiled reduction its embedding
    // surface applies), when it declares one. Idempotent and atomic;
    // legacy manifests without the field remain byte-identical.
    if let Some(geometry) = oracles[0].geometry_projection() {
        writer.set_geometry(&geometry)?;
    }

    // #602: record the typed attention-operator identity the teacher
    // oracle's source executor computes (the boolean `r4_attention`
    // switch resolved to its registered operator id), when the oracle
    // declares one. Idempotent and atomic; legacy manifests without the
    // field remain byte-identical.
    if let Some(operator) = oracles[0].attention_operator_spec() {
        writer.set_attention_operator(&operator)?;
    }

    // #601: record the versioned tokenizer-adapter identity this pass
    // segments with, when the selected tokenizer declares one (the HF
    // byte-level BPE path; the legacy llama2.c tokenizer declares none,
    // so legacy manifests remain byte-identical). Idempotent and atomic.
    if let Some(adapter) = tokenizer.adapter() {
        writer.set_tokenizer_adapter(&adapter)?;
    }

    let workers = oracles.len();
    let seq_len = oracles[0].seq_len();
    let mut progress = Progress::new("text observations", articles_total as usize);
    progress.set(checkpoint.stories as usize);
    let mut written = 0u64;
    let mut truncated = 0u64;
    let mut replaced = 0u64;
    let t0 = std::time::Instant::now();

    let file = fs::File::open(articles_path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", articles_path.display())))?;
    let mut lines = BufReader::new(file).lines();
    let mut ordinal = 0u64;
    let mut budget_hit = false;
    'batches: loop {
        // Gather up to `workers` not-yet-committed articles. Completed articles
        // (the dense committed prefix) are skipped; the budget stops the gather.
        let mut batch: Vec<(u64, Article)> = Vec::with_capacity(workers);
        while batch.len() < workers {
            let line = match lines.next() {
                Some(line) => line.map_err(|error| {
                    SourceUnavailable::new(format!("{}: {error}", articles_path.display()))
                })?,
                None => break,
            };
            if ordinal < checkpoint.stories {
                // Completed article: skip without re-deriving its records.
                ordinal += 1;
                continue;
            }
            if t0.elapsed().as_secs() >= budget_s {
                // This line is not processed; it is re-read on resume (the file
                // is re-scanned and the committed prefix skipped), so `ordinal`
                // is left un-advanced for it.
                budget_hit = true;
                break;
            }
            let article: Article = serde_json::from_str(&line).map_err(|error| {
                SourceUnavailable::new(format!(
                    "{} line {}: invalid article: {error}",
                    articles_path.display(),
                    ordinal + 1
                ))
            })?;
            batch.push((ordinal, article));
            ordinal += 1;
        }
        if batch.is_empty() {
            break;
        }
        // Produce the batch. A single-oracle pool runs on this thread and
        // threads `checkpoint.rng` exactly as the original loop (byte-identical,
        // checkpoint included). A multi-oracle pool teacher-forces the articles
        // in parallel — one worker thread per oracle — each with a local
        // throwaway rng, which is sound because no recorded field depends on it.
        let produced_batch: Vec<ArticleProduced> =
            if workers == 1 {
                let (ord, article) = &batch[0];
                vec![produce_article_records(
                    oracles[0].as_mut(),
                    *ord,
                    article,
                    seq_len,
                    tokenizer,
                    token_byte_lengths,
                    shard_bits,
                    &mut checkpoint.rng,
                )?]
            } else {
                std::thread::scope(|scope| -> Result<Vec<ArticleProduced>, SourceUnavailable> {
                    let mut handles = Vec::with_capacity(batch.len());
                    for ((ord, article), oracle) in batch.iter().zip(oracles.iter_mut()) {
                        handles.push(scope.spawn(move || {
                            let mut rng = RNG_SEED;
                            produce_article_records(
                                oracle.as_mut(),
                                *ord,
                                article,
                                seq_len,
                                tokenizer,
                                token_byte_lengths,
                                shard_bits,
                                &mut rng,
                            )
                        }));
                    }
                    let mut produced = Vec::with_capacity(handles.len());
                    for handle in handles {
                        produced.push(handle.join().map_err(|_| {
                            SourceUnavailable::new("observe worker thread panicked")
                        })??);
                    }
                    Ok(produced)
                })?
            };
        // Commit in ascending ordinal (the batch is gathered in order).
        for produced in &produced_batch {
            commit_article(
                &mut writer,
                &mut checkpoint,
                out_dir,
                &stories_path,
                produced,
                articles_total,
                &mut written,
                &mut truncated,
                &mut replaced,
            )?;
            progress.set((produced.ordinal + 1) as usize);
        }
        if budget_hit {
            break 'batches;
        }
    }
    finalize_text_observation(
        out_dir,
        &mut writer,
        &mut checkpoint,
        &mut progress,
        articles_total,
        ordinal,
        truncated,
        replaced,
        written,
    )
}

/// Finalize an observation: mark a fully-consumed stream done, κ-pin the shards,
/// and build the report. Shared by the serial and batched drivers.
#[allow(clippy::too_many_arguments)]
fn finalize_text_observation(
    out_dir: &Path,
    writer: &mut ObservationShardWriter,
    checkpoint: &mut Checkpoint,
    progress: &mut Progress,
    articles_total: u64,
    ordinal: u64,
    truncated: u64,
    replaced: u64,
    written: u64,
) -> Result<ObservationReport, SourceUnavailable> {
    if !checkpoint.done && ordinal == articles_total {
        // Empty (or fully consumed) stream: the corpus is complete.
        checkpoint.done = true;
        write_checkpoint(out_dir, checkpoint)?;
    }
    if checkpoint.done {
        writer.finalize_all()?;
        progress.finish();
    }
    let report = build_report(
        out_dir,
        checkpoint,
        writer,
        articles_total,
        truncated,
        replaced,
        written,
    )?;
    println!(
        "text observations: {} / {} articles, {} records ({} written), {}/{} shards complete, bits/token={:?}, done={}",
        report.articles_completed,
        report.articles_total,
        report.records,
        report.written,
        report.shards_completed,
        report.shard_count,
        report.teacher_bits_per_token,
        report.done
    );
    Ok(report)
}

/// One in-flight article in a batched observation group.
struct BatchSlot {
    ordinal: u64,
    article: Article,
    tokens: Vec<u32>,
    positions: usize,
    truncated: bool,
    replaced: u64,
    window: Vec<u32>,
    byte_off: u32,
    rng: u64,
    records: Vec<(u32, [u8; RECORD_SIZE], observe::ProbabilityMetadata)>,
}

/// Observe a text corpus with a batched teacher: up to `batch` articles are
/// teacher-forced together through the memory-amortized forward, so B articles
/// cost one weight sweep per step instead of B — the throughput lever (measured
/// ~15× at batch 32 on a 360M teacher). Articles are gathered in ordinal order
/// into a group, stepped together to the group's longest article (a finished
/// slot repeats its last position, which is idempotent, keeping the batch
/// contiguous), each active position encoded via the shared [`encode_position`],
/// then the group committed in ordinal order via [`commit_article`]. Records are
/// identical to the serial path for identical logits; on the deployed macOS
/// teacher the batched (`sgemm`) logits are a distinct, reproducible teacher
/// path from the serial (`sgemv`) one — both are teacher data, not the pinned
/// legacy proof.
#[allow(clippy::too_many_arguments)]
pub fn observe_text_corpus_batched(
    oracle: &dyn BatchedTeacher,
    batch: usize,
    budget_s: u64,
    tokenizer: &TokenizerKind,
    token_byte_lengths: Option<&[u32]>,
    articles_path: &Path,
    out_dir: &Path,
    shard_bits: u8,
    resume: bool,
) -> Result<ObservationReport, SourceUnavailable> {
    assert!(batch >= 1, "observe_text_corpus_batched needs batch >= 1");
    let (mut writer, mut checkpoint, articles_total, stories_path) =
        match prepare_text_observation(out_dir, articles_path, shard_bits, resume)? {
            Prepared::Done(report) => return Ok(report),
            Prepared::Ready {
                writer,
                checkpoint,
                articles_total,
                stories_path,
            } => (writer, checkpoint, articles_total, stories_path),
        };

    // #601: same tokenizer-adapter provenance as the serial driver — the
    // batched path segments with the identical `TokenizerKind` selection.
    if let Some(adapter) = tokenizer.adapter() {
        writer.set_tokenizer_adapter(&adapter)?;
    }

    let seq_len = oracle.seq_len();
    let mut progress = Progress::new("text observations", articles_total as usize);
    progress.set(checkpoint.stories as usize);
    let mut written = 0u64;
    let mut truncated = 0u64;
    let mut replaced = 0u64;
    let t0 = std::time::Instant::now();

    let file = fs::File::open(articles_path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", articles_path.display())))?;
    let mut lines = BufReader::new(file).lines();
    let mut ordinal = 0u64;
    let mut budget_hit = false;
    // One reusable state per slot, all sharing the oracle's single weight copy.
    let mut states: Vec<State> = (0..batch).map(|_| oracle.new_state()).collect();

    'groups: loop {
        let mut slots: Vec<BatchSlot> = Vec::with_capacity(batch);
        while slots.len() < batch {
            let line = match lines.next() {
                Some(line) => line.map_err(|error| {
                    SourceUnavailable::new(format!("{}: {error}", articles_path.display()))
                })?,
                None => break,
            };
            if ordinal < checkpoint.stories {
                ordinal += 1;
                continue;
            }
            if t0.elapsed().as_secs() >= budget_s {
                budget_hit = true;
                break;
            }
            let article: Article = serde_json::from_str(&line).map_err(|error| {
                SourceUnavailable::new(format!(
                    "{} line {}: invalid article: {error}",
                    articles_path.display(),
                    ordinal + 1
                ))
            })?;
            let (tokens, replaced_n) = tokenizer.encode_lossy(&article.text);
            let positions = tokens.len().saturating_sub(1).min(seq_len);
            let truncated = positions < tokens.len().saturating_sub(1);
            slots.push(BatchSlot {
                ordinal,
                article,
                tokens,
                positions,
                truncated,
                replaced: replaced_n,
                window: Vec::with_capacity(compiler::WINDOW),
                byte_off: 0,
                rng: RNG_SEED,
                records: Vec::new(),
            });
            ordinal += 1;
        }
        if slots.is_empty() {
            break;
        }
        let active = slots.len();
        states[..active].iter_mut().for_each(State::reset);
        let max_len = slots.iter().map(|s| s.positions).max().unwrap_or(0);

        for pos in 0..max_len {
            let tokens: Vec<usize> = slots
                .iter()
                .map(|s| {
                    let p = pos.min(s.positions.saturating_sub(1));
                    s.tokens.get(p).copied().unwrap_or(0) as usize
                })
                .collect();
            let positions: Vec<usize> = slots
                .iter()
                .map(|s| pos.min(s.positions.saturating_sub(1)))
                .collect();
            oracle.forward_batch_into(&mut states[..active], &tokens, &positions);
            for (i, slot) in slots.iter_mut().enumerate() {
                if pos < slot.positions {
                    let story = u32::try_from(slot.ordinal).map_err(|_| {
                        SourceUnavailable::new("article ordinal exceeds the u32 story field")
                    })?;
                    let token = slot.tokens[pos];
                    let next = slot.tokens[pos + 1];
                    let (encoded, advanced) = encode_position(
                        &mut states[i].logits,
                        story,
                        pos,
                        token,
                        next,
                        &mut slot.window,
                        slot.byte_off,
                        token_byte_lengths,
                        shard_bits,
                        &mut slot.rng,
                    );
                    slot.records
                        .push((encoded.shard, encoded.record, encoded.metadata));
                    slot.byte_off = advanced;
                }
            }
        }

        // Commit the group in ascending ordinal (slots gathered in order).
        for slot in &slots {
            let story = u32::try_from(slot.ordinal).map_err(|_| {
                SourceUnavailable::new("article ordinal exceeds the u32 story field")
            })?;
            let produced = ArticleProduced {
                ordinal: slot.ordinal,
                records: slot.records.clone(),
                story_entry: StoryEntry {
                    story,
                    id: slot.article.id.clone(),
                    url: slot.article.url.clone(),
                    title: slot.article.title.clone(),
                    partition: partition_of(&slot.article.id),
                },
                truncated: slot.truncated,
                replaced: slot.replaced,
            };
            commit_article(
                &mut writer,
                &mut checkpoint,
                out_dir,
                &stories_path,
                &produced,
                articles_total,
                &mut written,
                &mut truncated,
                &mut replaced,
            )?;
            progress.set((slot.ordinal + 1) as usize);
        }

        if budget_hit {
            break 'groups;
        }
    }

    finalize_text_observation(
        out_dir,
        &mut writer,
        &mut checkpoint,
        &mut progress,
        articles_total,
        ordinal,
        truncated,
        replaced,
        written,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{
        ObservationManifest, merge_shards, sample_id, shard_file_name, shard_of,
    };
    use std::time::{SystemTime, UNIX_EPOCH};
    use uor_r4_core::transformerless::scenarios::Tokenizer;
    use uor_r4_model_source::{BehaviorSource, RepresentationSource};

    const SHARD_BITS: u8 = 2;
    const SHARD_COUNT: u32 = 1 << SHARD_BITS;
    const FAKE_VOCAB: usize = 32;
    const FAKE_SEQ_LEN: usize = 16;

    // Tokenizer fixture pieces: byte fallback for ' ' and a..d plus four
    // merges; ids 1/2 stay the BOS/EOS convention.
    const PIECES: [&[u8]; 12] = [
        b"<unk>", b"<s>", b"</s>", b" ", b"a", b"b", b"c", b"d", b" a", b"ab", b"bc", b"cd",
    ];

    fn unique_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("uor-r4-observe-text-{name}-{nanos}"))
    }

    fn fixture_tokenizer() -> TokenizerKind {
        let path = unique_path("tokenizer.bin");
        let mut bytes = Vec::new();
        for piece in PIECES {
            bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            bytes.extend_from_slice(piece);
        }
        fs::write(&path, bytes).expect("write tokenizer fixture");
        let tokenizer = Tokenizer::try_load(&path).expect("load tokenizer fixture");
        let _ = fs::remove_file(&path);
        TokenizerKind::Legacy(tokenizer)
    }

    fn fixture_token_byte_lengths() -> Vec<u32> {
        PIECES.iter().map(|piece| piece.len() as u32).collect()
    }

    fn write_articles(path: &Path, articles: &[(&str, &str)]) {
        let mut bytes = Vec::new();
        for (id, text) in articles {
            let line = format!(
                "{{\"id\":\"{id}\",\"url\":\"https://example.test/{id}\",\"title\":\"Title {id}\",\"text\":\"{text}\"}}\n"
            );
            bytes.extend_from_slice(line.as_bytes());
        }
        fs::write(path, bytes).expect("write articles fixture");
    }

    #[test]
    fn encode_lossy_replaces_unencodable_characters_with_spaces() {
        let tokenizer = fixture_tokenizer();
        // 'Ɔ' (U+0186) is neither a whole token nor byte-encodable in the
        // fixture vocab (a..d and space only): the legacy llama2.c teacher
        // has exactly this gap for non-ASCII text (issue #72).
        let (tokens, replaced) = tokenizer.encode_lossy("abƆd");
        assert_eq!(replaced, 1);
        assert_eq!(tokens, tokenizer.encode("ab d"));
        // Fully encodable text is untouched.
        let (tokens, replaced) = tokenizer.encode_lossy("abcd");
        assert_eq!(replaced, 0);
        assert_eq!(tokens, tokenizer.encode("abcd"));
    }

    #[test]
    fn encode_byte_fallback_cannot_overflow_char_sized_buffer() {
        // Byte-level tokenizer (the HF path): a multi-byte character
        // decomposes into one token per byte, so a char-sized buffer
        // overflows — BOS + space + 4 byte tokens = 6 > 2 chars + 2
        // (issue #75). The buffer is sized by bytes.
        let pieces: [&[u8]; 6] = [b"<unk>", b"<s>", b"</s>", b" ", &[0xC6], &[0x86]];
        let path = unique_path("tokenizer-bytes.bin");
        let mut bytes = Vec::new();
        for piece in pieces {
            bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            bytes.extend_from_slice(piece);
        }
        fs::write(&path, bytes).expect("write tokenizer fixture");
        let tokenizer = Tokenizer::try_load(&path).expect("load tokenizer fixture");
        let _ = fs::remove_file(&path);
        // 'Ɔ' = U+0186 = bytes 0xC6 0x86; two of them decompose to four
        // byte tokens (no merge pieces here to recombine them).
        let tokens = tokenizer.encode("ƆƆ");
        assert_eq!(tokens.len(), 6);
    }

    /// Deterministic few-token oracle: logits depend only on (token, pos),
    /// so teacher-forced records are content-stable across restarts.
    struct FakeOracle;

    impl RepresentationSource for FakeOracle {
        fn vocab_size(&self) -> usize {
            FAKE_VOCAB
        }
        fn source_dimension(&self) -> usize {
            4
        }
        fn tokenizer_address(&self) -> &str {
            "fake-tokenizer"
        }
        fn read_embedding_rows(
            &self,
            _range: std::ops::Range<usize>,
            output: &mut [f32],
        ) -> Option<()> {
            output.fill(0.0);
            Some(())
        }
    }

    impl BehaviorSource for FakeOracle {
        fn reset(&mut self) {}
        fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
            for (index, logit) in logits.iter_mut().enumerate() {
                let value = (token as u64 * 31 + pos as u64 * 7 + index as u64 * 13) % 29;
                *logit = value as f32 * 0.25 - 3.0;
            }
        }
    }

    impl TeacherOracle for FakeOracle {
        fn vocab(&self) -> usize {
            FAKE_VOCAB
        }
        fn dim(&self) -> usize {
            4
        }
        fn seq_len(&self) -> usize {
            FAKE_SEQ_LEN
        }
        fn kappa(&self) -> String {
            "blake3:fake".to_owned()
        }
        fn source_bytes(&self) -> usize {
            0
        }
        fn embedding(&self, _token: usize, out: &mut [f32]) {
            out.fill(0.0);
        }
    }

    /// Independent replication of the driver loop over the first `up_to`
    /// articles, built ONLY from the shared encoder helpers: the expected
    /// per-shard record runs plus the rng state after them. This is the
    /// cross-check that the driver emits format-identical v3 bytes.
    fn expected_shards(
        articles: &[(&str, &str)],
        tokenizer: &TokenizerKind,
        token_byte_lengths: Option<&[u32]>,
        up_to: usize,
    ) -> (Vec<Vec<[u8; RECORD_SIZE]>>, u64) {
        let mut oracle = FakeOracle;
        let mut rng = RNG_SEED;
        let mut logits = vec![0f32; FAKE_VOCAB];
        let mut shards: Vec<Vec<[u8; RECORD_SIZE]>> =
            (0..SHARD_COUNT).map(|_| Vec::new()).collect();
        let mut window: Vec<u32> = Vec::new();
        for (ordinal, (_, text)) in articles.iter().enumerate().take(up_to) {
            let tokens = tokenizer.encode(text);
            let positions = tokens.len().saturating_sub(1).min(FAKE_SEQ_LEN);
            oracle.reset();
            window.clear();
            let mut offset = 0u32;
            for pos in 0..positions {
                let token = tokens[pos];
                oracle.step(token as usize, pos, &mut logits);
                let (_sampled, top_tokens, top_weights) =
                    compiler::softmax_top8_sample(&mut logits, &mut rng);
                let next = tokens[pos + 1];
                window.push(token);
                if window.len() > compiler::WINDOW {
                    window.remove(0);
                }
                let shard = shard_of(&sample_id(&window), SHARD_BITS);
                let (byte_start, byte_end) =
                    compiler::byte_anchors(token_byte_lengths, offset, next as usize);
                let record = compiler::encode_v4_record(
                    ordinal as u32,
                    next,
                    &top_tokens,
                    &top_weights,
                    (pos as u32, (pos as u32).saturating_add(1)),
                    (byte_start, byte_end),
                );
                shards[shard as usize].push(record);
                if token_byte_lengths.is_some() {
                    offset = byte_end;
                }
            }
        }
        (shards, rng)
    }

    fn expected_merged(
        articles: &[(&str, &str)],
        tokenizer: &TokenizerKind,
        token_byte_lengths: Option<&[u32]>,
    ) -> Vec<u8> {
        let (shards, _) = expected_shards(articles, tokenizer, token_byte_lengths, articles.len());
        shards.concat().concat()
    }

    /// Partition of each record in a shard file, via the story mapping.
    fn recount_partitions(dir: &Path, shard: u32, index: &StoryIndex) -> (u64, u64) {
        let bytes =
            fs::read(dir.join(shard_file_name(SHARD_BITS, shard))).expect("shard file bytes");
        let mut counts = (0u64, 0u64);
        for record in bytes.chunks_exact(RECORD_SIZE) {
            let story = u32::from_le_bytes(record[0..4].try_into().expect("story field"));
            match index.partition_of(story) {
                Some(RecordPartition::Construction) => counts.0 += 1,
                Some(RecordPartition::HeldOut) => counts.1 += 1,
                None => panic!("record story {story} missing from the story mapping"),
            }
        }
        counts
    }

    fn directory_fingerprint(dir: &Path) -> String {
        let mut hasher = blake3::Hasher::new();
        let mut entries: Vec<PathBuf> = fs::read_dir(dir)
            .expect("read dir")
            .map(|entry| entry.expect("dir entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            if entry.is_dir() {
                continue;
            }
            hasher.update(entry.file_name().expect("file name").as_encoded_bytes());
            hasher.update(&fs::read(&entry).expect("file bytes"));
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Six few-token articles with deterministic coverage of both
    /// partitions: the first four construction ids and the first two
    /// held-out ids found among "1".."=20".
    fn test_articles() -> Vec<(String, String)> {
        let texts = ["ab", "bc", "abcd", "ab bc", "", "cd ab"];
        let mut construction = Vec::new();
        let mut held_out = Vec::new();
        for ordinal in 1..=20u32 {
            let id = ordinal.to_string();
            match partition_of(&id) {
                RecordPartition::Construction if construction.len() < 4 => construction.push(id),
                RecordPartition::HeldOut if held_out.len() < 2 => held_out.push(id),
                _ => {}
            }
            if construction.len() == 4 && held_out.len() == 2 {
                break;
            }
        }
        assert_eq!(
            (construction.len(), held_out.len()),
            (4, 2),
            "partition fixture ids exhausted"
        );
        construction
            .into_iter()
            .chain(held_out)
            .zip(texts)
            .map(|(id, text)| (id, text.to_owned()))
            .collect()
    }

    #[test]
    fn partition_rule_is_blake3_first_byte_mod_5() {
        let mut held_out = 0usize;
        for ordinal in 1..=200u32 {
            let id = ordinal.to_string();
            let digest = blake3::hash(id.as_bytes());
            let expected = if digest.as_bytes()[0].is_multiple_of(5) {
                RecordPartition::HeldOut
            } else {
                RecordPartition::Construction
            };
            assert_eq!(partition_of(&id), expected, "article id {id}");
            if expected == RecordPartition::HeldOut {
                held_out += 1;
            }
        }
        assert!(held_out > 0 && held_out < 200, "both partitions populated");
    }

    #[test]
    fn checkpoint_roundtrip_and_state_layout() {
        let dir = unique_path("checkpoint");
        fs::create_dir_all(&dir).expect("mkdir");
        let kappa = *blake3::hash(b"articles").as_bytes();
        let mut checkpoint = Checkpoint::fresh(SHARD_COUNT, kappa);
        checkpoint.n = 3;
        checkpoint.stories = 2;
        checkpoint.rng = 0xABCD;
        checkpoint.shards[0].bytes = 2 * RECORD_SIZE as u64;
        checkpoint.shards[0].partitions = PartitionCounts {
            construction: 1,
            held_out: 1,
        };
        checkpoint.shards[3].bytes = RECORD_SIZE as u64;
        checkpoint.shards[3].partitions = PartitionCounts {
            construction: 0,
            held_out: 1,
        };
        write_checkpoint(&dir, &checkpoint).expect("write checkpoint");

        let committed = fs::read(dir.join(COMMITTED_FILE)).expect("committed.bin bytes");
        assert_eq!(
            committed.len(),
            HEADER_SIZE + INPUT_KAPPA_SIZE + SHARD_COUNT as usize * SHARD_ROW_SIZE
        );
        let decoded = Checkpoint::decode(&committed, SHARD_COUNT).expect("decode");
        assert_eq!(decoded, checkpoint);

        // state.bin mirrors the header in the exact 25-byte corpus-meta
        // layout: n u64 | stories u64 | rng u64 | done u8.
        let state = fs::read(dir.join(observe::STATE_FILE)).expect("state.bin bytes");
        assert_eq!(state.len(), 25);
        assert_eq!(u64::from_le_bytes(state[0..8].try_into().unwrap()), 3);
        assert_eq!(u64::from_le_bytes(state[8..16].try_into().unwrap()), 2);
        assert_eq!(
            u64::from_le_bytes(state[16..24].try_into().unwrap()),
            0xABCD
        );
        assert_eq!(state[24], 0);

        // A checkpoint whose record count disagrees with the shard lengths
        // is rejected.
        let mut torn = committed.clone();
        torn[0] = torn[0].wrapping_add(1);
        assert!(Checkpoint::decode(&torn, SHARD_COUNT).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn text_pipeline_records_shards_partitions_and_resume() {
        let articles = test_articles();
        let articles_ref: Vec<(&str, &str)> = articles
            .iter()
            .map(|(id, text)| (id.as_str(), text.as_str()))
            .collect();
        let tokenizer = fixture_tokenizer();
        let lengths = fixture_token_byte_lengths();
        let input = unique_path("articles.jsonl");
        write_articles(&input, &articles_ref);

        // Run A: single pass to completion.
        let dir_a = unique_path("run-a");
        let mut pool: Vec<Box<dyn TeacherOracle + Send>> = vec![Box::new(FakeOracle)];
        let report = observe_text_corpus(
            &mut pool,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir_a,
            SHARD_BITS,
            false,
        )
        .expect("single pass");
        assert!(report.done);
        assert_eq!(report.articles_total, articles.len() as u64);
        assert_eq!(report.articles_completed, articles.len() as u64);
        assert_eq!(report.shards_completed, SHARD_COUNT);
        assert_eq!(report.written, report.records);
        assert!(report.merged_kappa.is_some());
        assert!(report.teacher_bits_per_token.is_some());
        assert_eq!(
            report.construction_records + report.held_out_records,
            report.records
        );
        assert_eq!(
            report.construction_articles + report.held_out_articles,
            articles.len() as u64
        );

        // Record bytes cross-checked against the shared encoder: the
        // merged stream is exactly the per-shard ascending concatenation
        // of the independently replicated records.
        let expected = expected_merged(&articles_ref, &tokenizer, Some(&lengths));
        let merged = merge_shards(&dir_a).expect("merge a");
        assert_eq!(merged, expected, "driver bytes diverge from shared encoder");
        assert_eq!(merged.len() as u64, report.records * RECORD_SIZE as u64);
        let want_kappa = format!("blake3:{}", blake3::hash(&merged).to_hex());
        assert_eq!(report.merged_kappa.as_deref(), Some(want_kappa.as_str()));

        // Manifest: the rule is recorded and every shard entry's partition
        // counts match the rule applied to stories.jsonl.
        let manifest = ObservationManifest::load(&dir_a)
            .expect("manifest io")
            .expect("manifest");
        assert_eq!(manifest.partition_rule.as_deref(), Some(PARTITION_RULE));
        assert_eq!(manifest.total_records, report.records);
        let index = StoryIndex::load(&report.stories_file)
            .expect("story mapping io")
            .expect("story mapping");
        assert_eq!(index.len(), articles.len());
        for (ordinal, (id, _)) in articles.iter().enumerate() {
            let entry = index.get(ordinal as u32).expect("story entry");
            assert_eq!(entry.id, *id);
            assert_eq!(entry.partition, partition_of(id));
        }
        let (mut construction, mut held_out) = (0u64, 0u64);
        for shard in 0..SHARD_COUNT {
            let entry = manifest.completed.get(&shard).expect("shard entry");
            let partitions = entry.partitions.expect("partition counts");
            let (want_construction, want_held_out) = recount_partitions(&dir_a, shard, &index);
            assert_eq!(partitions.construction, want_construction, "shard {shard}");
            assert_eq!(partitions.held_out, want_held_out, "shard {shard}");
            assert_eq!(partitions.total(), entry.records, "shard {shard}");
            construction += partitions.construction;
            held_out += partitions.held_out;
        }
        assert_eq!(construction, report.construction_records);
        assert_eq!(held_out, report.held_out_records);
        // The fixture must actually exercise both partitions.
        assert!(construction > 0 && held_out > 0);

        // state.bin is the 25-byte corpus-meta header with done=1.
        let state = fs::read(dir_a.join(observe::STATE_FILE)).expect("state.bin");
        assert_eq!(state.len(), 25);
        assert_eq!(state[24], 1);
        assert_eq!(
            u64::from_le_bytes(state[8..16].try_into().unwrap()),
            articles.len() as u64
        );

        // A held-out-only merge contains exactly the records whose story
        // ids the rule marks held-out.
        let held_out_merged: Vec<&[u8]> = merged
            .chunks_exact(RECORD_SIZE)
            .filter(|record| {
                let story = u32::from_le_bytes(record[0..4].try_into().expect("story"));
                index.partition_of(story) == Some(RecordPartition::HeldOut)
            })
            .collect();
        assert_eq!(held_out_merged.len() as u64, held_out);

        // Rerun: fully resumed, no byte changes anywhere.
        let fingerprint = directory_fingerprint(&dir_a);
        let rerun = observe_text_corpus(
            &mut pool,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir_a,
            SHARD_BITS,
            true,
        )
        .expect("idempotent rerun");
        assert!(rerun.done);
        assert_eq!(rerun.written, 0);
        assert_eq!(rerun.records, report.records);
        assert_eq!(rerun.merged_kappa, report.merged_kappa);
        assert_eq!(
            directory_fingerprint(&dir_a),
            fingerprint,
            "completed observation directory changed on rerun"
        );
        // resume=false refuses a non-empty directory.
        assert!(
            observe_text_corpus(
                &mut pool,
                60,
                &tokenizer,
                Some(&lengths),
                &input,
                &dir_a,
                SHARD_BITS,
                false,
            )
            .is_err()
        );

        // Run B: budget-starved first invocation, then resumed — merged
        // bytes are identical to the single-pass run (T-invariance across
        // article completion order).
        let dir_b = unique_path("run-b");
        let starved = observe_text_corpus(
            &mut pool,
            0,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir_b,
            SHARD_BITS,
            true,
        )
        .expect("budget-starved pass");
        assert!(!starved.done);
        assert_eq!(starved.written, 0);
        let resumed = observe_text_corpus(
            &mut pool,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir_b,
            SHARD_BITS,
            true,
        )
        .expect("resumed pass");
        assert!(resumed.done);
        assert_eq!(merge_shards(&dir_b).expect("merge b"), expected);

        for dir in [&dir_a, &dir_b] {
            let _ = fs::remove_dir_all(dir);
        }
        let _ = fs::remove_file(&input);
    }

    #[test]
    fn crash_trims_converge_to_single_pass_kappa() {
        let articles = test_articles();
        let articles_ref: Vec<(&str, &str)> = articles
            .iter()
            .map(|(id, text)| (id.as_str(), text.as_str()))
            .collect();
        let tokenizer = fixture_tokenizer();
        let lengths = fixture_token_byte_lengths();
        let input = unique_path("articles.jsonl");
        write_articles(&input, &articles_ref);
        let expected = expected_merged(&articles_ref, &tokenizer, Some(&lengths));
        let (expected_shards_all, _) =
            expected_shards(&articles_ref, &tokenizer, Some(&lengths), articles.len());
        let (article0_shards, rng_after_0) =
            expected_shards(&articles_ref, &tokenizer, Some(&lengths), 1);

        // Craft A: a crash before the first checkpoint — shard files hold
        // a partial article-0 tail and one story line, no committed.bin.
        // Open must trim everything and recompute from scratch.
        let dir_a = unique_path("crash-pre");
        fs::create_dir_all(&dir_a).expect("mkdir a");
        let index_path = dir_a.join(STORIES_FILE);
        for shard in 0..SHARD_COUNT {
            let records = &article0_shards[shard as usize];
            let partial: Vec<u8> = records[..records.len() / 2].concat();
            fs::write(dir_a.join(shard_file_name(SHARD_BITS, shard)), partial)
                .expect("craft shard tail");
        }
        append_story(
            &index_path,
            &StoryEntry {
                story: 0,
                id: articles[0].0.clone(),
                url: format!("https://example.test/{}", articles[0].0),
                title: format!("Title {}", articles[0].0),
                partition: partition_of(&articles[0].0),
            },
        )
        .expect("craft story line");
        let mut pool: Vec<Box<dyn TeacherOracle + Send>> = vec![Box::new(FakeOracle)];
        let report_a = observe_text_corpus(
            &mut pool,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir_a,
            SHARD_BITS,
            true,
        )
        .expect("pre-checkpoint crash recovery");
        assert!(report_a.done);
        assert_eq!(merge_shards(&dir_a).expect("merge a"), expected);

        // Craft B: a crash after the article-0 checkpoint — committed.bin
        // pins article 0 but shard files and stories.jsonl already hold
        // later articles' content. Open must trim both back to the
        // checkpoint and recompute the trimmed articles exactly once.
        let dir_b = unique_path("crash-post");
        fs::create_dir_all(&dir_b).expect("mkdir b");
        {
            // Manifest with the partition rule, as a real first pass
            // leaves it; no shards finalized.
            let mut writer =
                ObservationShardWriter::open(&dir_b, SHARD_BITS).expect("open craft writer");
            writer.set_partition_rule(PARTITION_RULE).expect("rule");
        }
        for shard in 0..SHARD_COUNT {
            let mut bytes = article0_shards[shard as usize].concat();
            // The tail: every later article's records for this shard.
            let tail: Vec<[u8; RECORD_SIZE]> = expected_shards_all[shard as usize]
                [article0_shards[shard as usize].len()..]
                .to_vec();
            bytes.extend_from_slice(&tail.concat());
            fs::write(dir_b.join(shard_file_name(SHARD_BITS, shard)), bytes)
                .expect("craft shard bytes");
            fs::copy(
                dir_a.join(format!("{}.prob", shard_file_name(SHARD_BITS, shard))),
                dir_b.join(format!("{}.prob", shard_file_name(SHARD_BITS, shard))),
            )
            .expect("craft probability sidecar");
        }
        let mut committed =
            Checkpoint::fresh(SHARD_COUNT, input_kappa(&input).expect("input kappa"));
        committed.n = article0_shards.iter().map(Vec::len).sum::<usize>() as u64;
        committed.stories = 1;
        committed.rng = rng_after_0;
        for shard in 0..SHARD_COUNT {
            let records = &article0_shards[shard as usize];
            let partition = partition_of(&articles[0].0);
            let mut counts = PartitionCounts::default();
            for _ in records {
                match partition {
                    RecordPartition::Construction => counts.construction += 1,
                    RecordPartition::HeldOut => counts.held_out += 1,
                }
            }
            committed.shards[shard as usize] = ShardCheckpoint {
                bytes: (records.len() * RECORD_SIZE) as u64,
                partitions: counts,
            };
        }
        write_checkpoint(&dir_b, &committed).expect("craft checkpoint");
        // stories.jsonl holds both committed story 0 and uncommitted
        // story 1.
        for (ordinal, (id, _)) in articles.iter().enumerate().take(2) {
            append_story(
                &dir_b.join(STORIES_FILE),
                &StoryEntry {
                    story: ordinal as u32,
                    id: id.clone(),
                    url: format!("https://example.test/{id}"),
                    title: format!("Title {id}"),
                    partition: partition_of(id),
                },
            )
            .expect("craft story line");
        }
        let report_b = observe_text_corpus(
            &mut pool,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir_b,
            SHARD_BITS,
            true,
        )
        .expect("post-checkpoint crash recovery");
        assert!(report_b.done);
        assert_eq!(merge_shards(&dir_b).expect("merge b"), expected);
        let index = StoryIndex::load(&report_b.stories_file)
            .expect("story mapping io")
            .expect("story mapping");
        assert_eq!(index.len(), articles.len());

        for dir in [&dir_a, &dir_b] {
            let _ = fs::remove_dir_all(dir);
        }
        let _ = fs::remove_file(&input);
    }

    #[test]
    fn unknown_byte_anchors_and_sequence_length_truncation() {
        let tokenizer = fixture_tokenizer();
        let input = unique_path("articles.jsonl");
        // One long article with no mergeable pairs (41 tokens, exceeding
        // the 16-position teacher window) and one short one; no token
        // byte lengths → v3 "unknown" anchors.
        const LONG_TEXT: &str = "adadadadadadadadadadadadadadadadadadadad";
        write_articles(&input, &[("9", LONG_TEXT), ("10", "ab")]);
        let dir = unique_path("anchors");
        let mut pool: Vec<Box<dyn TeacherOracle + Send>> = vec![Box::new(FakeOracle)];
        let report = observe_text_corpus(
            &mut pool, 60, &tokenizer, None, &input, &dir, SHARD_BITS, false,
        )
        .expect("pass");
        assert!(report.done);
        assert_eq!(report.articles_truncated, 1);
        // The long article contributes exactly seq_len records.
        let long_tokens = tokenizer.encode(LONG_TEXT);
        assert!(long_tokens.len() - 1 > FAKE_SEQ_LEN);
        let merged = merge_shards(&dir).expect("merge");
        let long_records = merged
            .chunks_exact(RECORD_SIZE)
            .filter(|record| record[0..4] == 0u32.to_le_bytes())
            .count();
        assert_eq!(long_records, FAKE_SEQ_LEN);
        for record in merged.chunks_exact(RECORD_SIZE) {
            assert_eq!(&record[80..84], &u32::MAX.to_le_bytes());
            assert_eq!(&record[84..88], &u32::MAX.to_le_bytes());
        }
        // The same replication check holds on the unknown-anchor path.
        let expected = expected_merged(&[("9", LONG_TEXT), ("10", "ab")], &tokenizer, None);
        assert_eq!(merged, expected);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(&input);
    }

    #[test]
    fn merged_records_load_as_v3_corpus() {
        let articles = test_articles();
        let articles_ref: Vec<(&str, &str)> = articles
            .iter()
            .map(|(id, text)| (id.as_str(), text.as_str()))
            .collect();
        let tokenizer = fixture_tokenizer();
        let lengths = fixture_token_byte_lengths();
        let input = unique_path("articles.jsonl");
        write_articles(&input, &articles_ref);
        let dir = unique_path("corpus-load");
        let mut pool: Vec<Box<dyn TeacherOracle + Send>> = vec![Box::new(FakeOracle)];
        let report = observe_text_corpus(
            &mut pool,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir,
            SHARD_BITS,
            false,
        )
        .expect("pass");
        assert!(report.done);

        let merged = merge_shards(&dir).expect("merge");
        let meta = unique_path("corpus.meta");
        let recs = unique_path("corpus.records");
        let mut header = [0u8; 25];
        header[0..8].copy_from_slice(&report.records.to_le_bytes());
        header[8..16].copy_from_slice(&(articles.len() as u64).to_le_bytes());
        header[16..24].copy_from_slice(&RNG_SEED.to_le_bytes());
        header[24] = 1;
        fs::write(&meta, header).expect("meta");
        fs::write(&recs, &merged).expect("recs");
        let corpus = compiler::load_corpus_from(
            meta.to_str().expect("meta utf-8"),
            recs.to_str().expect("recs utf-8"),
        )
        .expect("merged observation records must parse as a v3 corpus");
        assert_eq!(corpus.n, report.records as usize);
        // Cross-check story/span/anchor fields against the replication.
        let (shards, _) =
            expected_shards(&articles_ref, &tokenizer, Some(&lengths), articles.len());
        let expected = shards.concat();
        for (index, record) in expected.concat().chunks_exact(RECORD_SIZE).enumerate() {
            let story = u32::from_le_bytes(record[0..4].try_into().unwrap());
            let next = u32::from_le_bytes(record[4..8].try_into().unwrap());
            let byte_start = u32::from_le_bytes(record[80..84].try_into().unwrap());
            assert_eq!(corpus.story[index], story);
            assert_eq!(corpus.next[index], next);
            assert_eq!(corpus.byte_start[index], byte_start);
        }
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(&input);
        let _ = fs::remove_file(&meta);
        let _ = fs::remove_file(&recs);
    }

    /// A multi-worker pool must produce a byte-for-byte identical corpus to a
    /// single worker: articles are teacher-forced independently and committed
    /// in ascending ordinal, so worker count changes only *how fast* the
    /// records are produced, never *what* they are. Guards the parallel observe
    /// path added for #531.
    #[test]
    fn parallel_workers_match_single_worker_byte_for_byte() {
        let articles = test_articles();
        let articles_ref: Vec<(&str, &str)> = articles
            .iter()
            .map(|(id, text)| (id.as_str(), text.as_str()))
            .collect();
        let tokenizer = fixture_tokenizer();
        let lengths = fixture_token_byte_lengths();
        let input = unique_path("articles.jsonl");
        write_articles(&input, &articles_ref);

        let dir1 = unique_path("workers-1");
        let mut pool1: Vec<Box<dyn TeacherOracle + Send>> = vec![Box::new(FakeOracle)];
        let report1 = observe_text_corpus(
            &mut pool1,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir1,
            SHARD_BITS,
            false,
        )
        .expect("workers=1");
        assert!(report1.done);

        let dir3 = unique_path("workers-3");
        let mut pool3: Vec<Box<dyn TeacherOracle + Send>> = vec![
            Box::new(FakeOracle),
            Box::new(FakeOracle),
            Box::new(FakeOracle),
        ];
        let report3 = observe_text_corpus(
            &mut pool3,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir3,
            SHARD_BITS,
            false,
        )
        .expect("workers=3");
        assert!(report3.done);

        // The merged record stream, its κ, every shard file, and the story
        // mapping are all identical regardless of worker count.
        let merged1 = merge_shards(&dir1).expect("merge workers=1");
        let merged3 = merge_shards(&dir3).expect("merge workers=3");
        assert_eq!(
            merged1, merged3,
            "parallel merged bytes diverge from single worker"
        );
        assert_eq!(report1.merged_kappa, report3.merged_kappa);
        assert_eq!(report1.records, report3.records);
        assert_eq!(report1.written, report3.written);
        for shard in 0..SHARD_COUNT {
            let f1 = fs::read(dir1.join(shard_file_name(SHARD_BITS, shard))).unwrap_or_default();
            let f3 = fs::read(dir3.join(shard_file_name(SHARD_BITS, shard))).unwrap_or_default();
            assert_eq!(f1, f3, "shard {shard} bytes differ across worker counts");
        }
        let stories1 = fs::read_to_string(&report1.stories_file).expect("stories workers=1");
        let stories3 = fs::read_to_string(&report3.stories_file).expect("stories workers=3");
        assert_eq!(
            stories1, stories3,
            "story mapping differs across worker counts"
        );

        let _ = fs::remove_dir_all(&dir1);
        let _ = fs::remove_dir_all(&dir3);
        let _ = fs::remove_file(&input);
    }

    /// A batched teacher whose per-position logits match `FakeOracle::step`
    /// exactly, so the batched driver's records can be compared byte-for-byte
    /// against the serial driver's.
    struct FakeBatchedOracle {
        cfg: uor_r4_model_source::Config,
    }

    impl BatchedTeacher for FakeBatchedOracle {
        fn new_state(&self) -> State {
            State::new(&self.cfg)
        }
        fn seq_len(&self) -> usize {
            FAKE_SEQ_LEN
        }
        fn vocab(&self) -> usize {
            FAKE_VOCAB
        }
        fn forward_batch_into(&self, states: &mut [State], tokens: &[usize], positions: &[usize]) {
            for (b, st) in states.iter_mut().enumerate() {
                let (token, pos) = (tokens[b], positions[b]);
                for (index, logit) in st.logits.iter_mut().enumerate() {
                    let value = (token as u64 * 31 + pos as u64 * 7 + index as u64 * 13) % 29;
                    *logit = value as f32 * 0.25 - 3.0;
                }
            }
        }
    }

    fn fake_batched_config() -> uor_r4_model_source::Config {
        uor_r4_model_source::Config {
            dim: 4,
            hidden: 4,
            n_layers: 1,
            n_heads: 2,
            n_kv_heads: 2,
            vocab: FAKE_VOCAB,
            seq_len: FAKE_SEQ_LEN,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            rope_interleaved: true,
            r4_attention: false,
        }
    }

    /// The batched driver must produce a byte-for-byte identical corpus to the
    /// serial driver when the teacher's per-position logits match: a fake whose
    /// logits equal FakeOracle's is observed serially and with batch=4, and the
    /// merged stream, its κ, every shard, and the story mapping must be equal.
    /// Guards the batched observe path added for #531.
    #[test]
    fn batched_matches_serial_byte_for_byte() {
        let articles = test_articles();
        let articles_ref: Vec<(&str, &str)> = articles
            .iter()
            .map(|(id, text)| (id.as_str(), text.as_str()))
            .collect();
        let tokenizer = fixture_tokenizer();
        let lengths = fixture_token_byte_lengths();
        let input = unique_path("articles.jsonl");
        write_articles(&input, &articles_ref);

        let dir_s = unique_path("serial");
        let mut pool: Vec<Box<dyn TeacherOracle + Send>> = vec![Box::new(FakeOracle)];
        let serial = observe_text_corpus(
            &mut pool,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir_s,
            SHARD_BITS,
            false,
        )
        .expect("serial observe");
        assert!(serial.done);

        let dir_b = unique_path("batched");
        let fake = FakeBatchedOracle {
            cfg: fake_batched_config(),
        };
        let batched = observe_text_corpus_batched(
            &fake,
            4,
            60,
            &tokenizer,
            Some(&lengths),
            &input,
            &dir_b,
            SHARD_BITS,
            false,
        )
        .expect("batched observe");
        assert!(batched.done);

        assert_eq!(
            merge_shards(&dir_s).expect("merge serial"),
            merge_shards(&dir_b).expect("merge batched"),
            "batched merged bytes differ from serial"
        );
        assert_eq!(serial.merged_kappa, batched.merged_kappa);
        assert_eq!(serial.records, batched.records);
        assert_eq!(serial.written, batched.written);
        for shard in 0..SHARD_COUNT {
            let f_s = fs::read(dir_s.join(shard_file_name(SHARD_BITS, shard))).unwrap_or_default();
            let f_b = fs::read(dir_b.join(shard_file_name(SHARD_BITS, shard))).unwrap_or_default();
            assert_eq!(f_s, f_b, "shard {shard} differs between serial and batched");
        }
        assert_eq!(
            fs::read_to_string(&serial.stories_file).expect("serial stories"),
            fs::read_to_string(&batched.stories_file).expect("batched stories"),
            "story mapping differs between serial and batched"
        );

        let _ = fs::remove_dir_all(&dir_s);
        let _ = fs::remove_dir_all(&dir_b);
        let _ = fs::remove_file(&input);
    }
}
