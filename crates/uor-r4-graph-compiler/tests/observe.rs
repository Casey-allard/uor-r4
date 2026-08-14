//! Observation pipeline v2 tests (graph-compiler plan §4.1 / §5 Phase 2):
//! content-addressed sample ids, deterministic shard partitioning, spill +
//! manifest + resume, ordered merge (T-invariance), the optional teacher
//! trace surface, and the `observe` CLI.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};
use uor_r4_graph_compiler::observation::{
    MANIFEST_FILE, ObservationManifest, ObservationShardWriter, ProbabilityMetadata, RECORD_SIZE,
    STATE_FILE, merge_probability_metadata, merge_shards, merge_trace_rows, message_bits_per_token,
    observe_sharded, observe_sharded_traced, sample_id, shard_file_name, shard_of,
    trace_sidecar_name,
};
use uor_r4_graph_compiler::trace_profile::{SUPPORT_ABSENT_MARKER, TraceProfile};
use uor_r4_model_source::geometry::GeometryProjection;
use uor_r4_model_source::{
    BehaviorSource, LlamaOracle, RepresentationSource, TeacherOracle, TraceCaptureGeometry,
    TraceCaptureRequest, TraceCaptureSinks,
};

const LEGACY_CHECKPOINT: &str = "/tmp/ref/out/model.bin";

fn unique_path(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("uor-r4-{name}-{nanos}"))
}

fn kappa_of(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn directory_bytes(dir: &std::path::Path) -> BTreeMap<String, Vec<u8>> {
    std::fs::read_dir(dir)
        .expect("read observation directory")
        .map(|entry| {
            let entry = entry.expect("read directory entry");
            let name = entry
                .file_name()
                .into_string()
                .expect("observation file name is UTF-8");
            let bytes = std::fs::read(entry.path()).expect("read observation file");
            (name, bytes)
        })
        .collect()
}

fn fixture_byte_bpe_adapter(
    tokenizer_json: &str,
) -> uor_r4_core::transformerless::hf_bpe::TokenizerAdapter {
    uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer::from_tokenizer_json_bytes(
        tokenizer_json.as_bytes(),
    )
    .expect("fixture tokenizer parses")
    .adapter()
}

// ------------------------------------------------------------ sample id --

#[test]
fn sample_id_is_blake3_over_little_endian_token_bytes() {
    let tokens = [1u32, 2, 3, 4, 5, 6, 7, 8];
    let mut bytes = Vec::new();
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    assert_eq!(sample_id(&tokens), *blake3::hash(&bytes).as_bytes());
    assert_eq!(sample_id(&[]), *blake3::hash(&[]).as_bytes());
}

#[test]
fn sample_id_is_content_addressed() {
    let tokens = [11u32, 22, 33, 44];
    assert_eq!(sample_id(&tokens), sample_id(&tokens));
    assert_ne!(sample_id(&tokens), sample_id(&[11, 22, 33, 45]));
    assert_ne!(sample_id(&tokens), sample_id(&tokens[..3]));
    assert_ne!(sample_id(&tokens), sample_id(&[44, 33, 22, 11]));
}

// ------------------------------------------------------------- shard_of --

#[test]
fn shard_of_reads_big_endian_prefix_bits() {
    let mut id = [0u8; 32];
    id[0] = 0b1011_0011;
    id[1] = 0b1110_0000;
    assert_eq!(shard_of(&id, 0), 0);
    assert_eq!(shard_of(&id, 1), 0b1);
    assert_eq!(shard_of(&id, 4), 0b1011);
    assert_eq!(shard_of(&id, 8), 0b1011_0011);
    assert_eq!(shard_of(&id, 12), 0b1011_0011_1110);
    assert_eq!(shard_of(&[0u8; 32], 8), 0);
}

#[test]
fn shard_of_partitions_into_exact_fanout() {
    for bits in [0u8, 1, 4, 8] {
        let mut buckets = BTreeSet::new();
        for first in 0..=255u8 {
            let mut id = [0u8; 32];
            id[0] = first;
            let shard = shard_of(&id, bits);
            assert!(shard < (1u32 << bits));
            buckets.insert(shard);
        }
        assert_eq!(buckets.len(), 1usize << bits, "shard_bits={bits}");
        assert_eq!(*buckets.iter().next_back().unwrap(), (1u32 << bits) - 1);
    }
    let id = sample_id(&[42, 43, 44]);
    assert_eq!(shard_of(&id, 5), shard_of(&id, 5), "same id, same shard");
}

// ------------------------------------------- spill / manifest / resume --

const SHARD_BITS: u8 = 3;
const SHARD_COUNT: u32 = 1 << SHARD_BITS;
const RECORD_COUNT: usize = 400;

fn synth_records() -> Vec<[u8; RECORD_SIZE]> {
    (0..RECORD_COUNT)
        .map(|i| {
            let mut record = [0u8; RECORD_SIZE];
            record[0..4].copy_from_slice(&(i as u32).to_le_bytes());
            for (j, byte) in record[4..].iter_mut().enumerate() {
                *byte = ((i * 31 + j * 7) % 251) as u8;
            }
            record
        })
        .collect()
}

/// Shard assignment mirroring the pipeline: sample id over an 8-token
/// context window derived from the record index.
fn record_shard(i: usize) -> u32 {
    let context = [i as u32, 0xC0FFEE, 6, 5, 4, 3, 2, 1];
    shard_of(&sample_id(&context), SHARD_BITS)
}

fn group_by_shard(records: &[[u8; RECORD_SIZE]]) -> Vec<Vec<[u8; RECORD_SIZE]>> {
    let mut groups = vec![Vec::new(); SHARD_COUNT as usize];
    for (i, record) in records.iter().enumerate() {
        groups[record_shard(i) as usize].push(*record);
    }
    groups
}

/// Merged bytes are per-shard record runs in ascending shard-id order.
fn expected_merged(groups: &[Vec<[u8; RECORD_SIZE]>]) -> Vec<u8> {
    let mut expected = Vec::new();
    for group in groups {
        for record in group {
            expected.extend_from_slice(record);
        }
    }
    expected
}

fn assert_manifest_kappas(dir: &std::path::Path, manifest: &ObservationManifest) {
    for shard in 0..SHARD_COUNT {
        let bytes = std::fs::read(dir.join(shard_file_name(SHARD_BITS, shard))).expect("shard");
        let entry = manifest.completed.get(&shard).expect("completed entry");
        assert_eq!(entry.records, bytes.len() as u64 / RECORD_SIZE as u64);
        assert_eq!(entry.kappa, kappa_of(&bytes), "shard {shard} κ");
    }
}

#[test]
fn shard_spill_manifest_resume_and_merge() {
    let records = synth_records();
    let groups = group_by_shard(&records);
    assert!(
        groups[3].len() >= 2,
        "deterministic fixture must give shard 3 a splittable prefix"
    );
    let expected = expected_merged(&groups);

    // Run A: one fresh pass, all shards finalized at once.
    let dir_a = unique_path("observe-a");
    let mut writer = ObservationShardWriter::open(&dir_a, SHARD_BITS).expect("open a");
    for (i, record) in records.iter().enumerate() {
        assert!(writer.write_record(record, record_shard(i)).expect("write"));
    }
    writer.finalize_all().expect("finalize a");
    let manifest_a = ObservationManifest::load(&dir_a)
        .expect("load a")
        .expect("manifest a");
    assert_eq!(manifest_a.shard_bits, SHARD_BITS);
    assert_eq!(manifest_a.completed.len(), SHARD_COUNT as usize);
    assert_eq!(manifest_a.total_records, RECORD_COUNT as u64);
    assert_manifest_kappas(&dir_a, &manifest_a);
    assert_eq!(merge_shards(&dir_a).expect("merge a"), expected);

    // Run B: shards 0..2 finalized, shard 3 half-written, then a "crash"
    // (the writer is dropped without finalizing). Resume must complete
    // exactly the missing five shards and never rewrite completed ones.
    let dir_b = unique_path("observe-b");
    let partial3 = groups[3].len() / 2;
    {
        let mut writer = ObservationShardWriter::open(&dir_b, SHARD_BITS).expect("open b");
        for (shard, group) in groups.iter().enumerate().take(3) {
            for record in group {
                assert!(writer.write_record(record, shard as u32).expect("write"));
            }
            writer.finish_shard(shard as u32).expect("finish");
            assert!(writer.is_complete(shard as u32));
        }
        for record in groups[3].iter().take(partial3) {
            writer.write_record(record, 3).expect("partial write");
        }
    }
    let shard0_before =
        std::fs::read(dir_b.join(shard_file_name(SHARD_BITS, 0))).expect("shard 0 bytes");
    let mut writer = ObservationShardWriter::open(&dir_b, SHARD_BITS).expect("reopen b");
    let completed: Vec<u32> = writer.manifest().completed.keys().copied().collect();
    assert_eq!(completed, vec![0, 1, 2], "crash survivors");
    let expected_total: u64 = groups[..3].iter().map(|group| group.len() as u64).sum();
    assert_eq!(writer.manifest().total_records, expected_total);
    // A record routed to a completed shard is skipped, not rewritten.
    assert!(
        !writer
            .write_record(&groups[0][0], 0)
            .expect("skip completed")
    );
    let shard0_after =
        std::fs::read(dir_b.join(shard_file_name(SHARD_BITS, 0))).expect("shard 0 bytes");
    assert_eq!(shard0_before, shard0_after, "completed shard rewritten");
    // Complete exactly the missing five shards; shard 3 resumes from its
    // on-disk partial prefix.
    for shard in 3..SHARD_COUNT {
        let start = if shard == 3 { partial3 } else { 0 };
        for record in groups[shard as usize].iter().skip(start) {
            assert!(writer.write_record(record, shard).expect("resume write"));
        }
        writer.finish_shard(shard).expect("resume finish");
    }
    assert_eq!(writer.manifest().completed.len(), SHARD_COUNT as usize);
    assert_eq!(writer.manifest().total_records, RECORD_COUNT as u64);
    assert_manifest_kappas(&dir_b, writer.manifest());
    assert_eq!(
        merge_shards(&dir_b).expect("merge b"),
        expected,
        "resumed run must merge to the same bytes as the fresh run"
    );

    // Run C: identical records, shards finalized in reverse order —
    // completion order must not change the merged bytes (T-invariance).
    let dir_c = unique_path("observe-c");
    let mut writer = ObservationShardWriter::open(&dir_c, SHARD_BITS).expect("open c");
    for (i, record) in records.iter().enumerate() {
        writer
            .write_record(record, record_shard(i))
            .expect("write c");
    }
    for shard in (0..SHARD_COUNT).rev() {
        writer.finish_shard(shard).expect("finish c");
    }
    assert_eq!(
        merge_shards(&dir_c).expect("merge c"),
        expected,
        "shard completion order changed merged bytes"
    );

    for dir in [&dir_a, &dir_b, &dir_c] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn probability_sidecar_is_aligned_and_reports_message_bits() {
    let dir = unique_path("observe-probability");
    let records = synth_records();
    let mut writer = ObservationShardWriter::open(&dir, SHARD_BITS).expect("open");
    for (i, record) in records.iter().take(4).enumerate() {
        assert!(
            writer
                .write_record_with_probability(
                    record,
                    ProbabilityMetadata {
                        target_logprob_nats: -0.5 - i as f32,
                        entropy_bits: 2.0 + i as f32,
                        top8_mass: 0.75,
                        target_rank: i as u16,
                    },
                    record_shard(i),
                )
                .expect("write probability")
        );
    }
    writer.finalize_all().expect("finalize");
    let metadata = merge_probability_metadata(&dir).expect("merge probability metadata");
    assert_eq!(metadata.len(), 4);
    assert!(metadata.iter().all(|row| row.top8_mass == 0.75));
    let mut ranks: Vec<u16> = metadata.iter().map(|row| row.target_rank).collect();
    ranks.sort_unstable();
    assert_eq!(ranks, vec![0, 1, 2, 3]);
    let bits = message_bits_per_token(&metadata).expect("non-empty message");
    let expected = (0.5f64 + 1.5 + 2.5 + 3.5) / std::f64::consts::LN_2 / 4.0;
    assert!(
        (bits - expected).abs() < 1e-6,
        "bits/token={bits}, expected={expected}"
    );
    let manifest = ObservationManifest::load(&dir)
        .expect("load manifest")
        .expect("manifest");
    assert!(
        manifest
            .completed
            .values()
            .filter(|entry| entry.records != 0)
            .all(|entry| entry.probability_kappa.is_some())
    );
    let _ = std::fs::remove_dir_all(dir);
}

// -------------------------------------------------------- trace surface --

struct FakeOracle {
    dim: usize,
    vocab: usize,
}

impl RepresentationSource for FakeOracle {
    fn vocab_size(&self) -> usize {
        self.vocab
    }
    fn source_dimension(&self) -> usize {
        self.dim
    }
    fn tokenizer_address(&self) -> &str {
        "fake-tokenizer"
    }
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()> {
        for (i, value) in output.iter_mut().enumerate() {
            *value = (range.start + i) as f32;
        }
        Some(())
    }
}

impl BehaviorSource for FakeOracle {
    fn reset(&mut self) {}
    fn step(&mut self, _token: usize, _pos: usize, logits: &mut [f32]) {
        for (i, logit) in logits.iter_mut().enumerate() {
            *logit = i as f32;
        }
    }
}

impl TeacherOracle for FakeOracle {
    fn vocab(&self) -> usize {
        self.vocab
    }
    fn dim(&self) -> usize {
        self.dim
    }
    fn seq_len(&self) -> usize {
        16
    }
    fn kappa(&self) -> String {
        "blake3:fake".to_string()
    }
    fn source_bytes(&self) -> usize {
        0
    }
    fn embedding(&self, _token: usize, out: &mut [f32]) {
        for value in out.iter_mut() {
            *value = 0.0;
        }
    }
}

#[test]
fn trace_surface_defaults_to_none_and_zero() {
    let oracle = FakeOracle { dim: 4, vocab: 8 };
    assert!(oracle.hidden_state().is_none());
    let mut out = [(0u32, 0f32); 4];
    assert_eq!(oracle.top_k(4, &mut out), 0);
}

#[test]
fn llama_oracle_exposes_hidden_state_and_canonical_top_k() {
    if std::fs::metadata(LEGACY_CHECKPOINT).is_err() {
        eprintln!("skipping: source checkpoint not found at {LEGACY_CHECKPOINT}");
        return;
    }
    let mut oracle = LlamaOracle::load(LEGACY_CHECKPOINT);
    oracle.reset();
    let bos = oracle.bos_token();
    let mut logits = vec![0f32; oracle.vocab()];
    oracle.step(bos, 0, &mut logits);

    let hidden = oracle
        .hidden_state()
        .expect("llama oracle retains the final hidden state");
    assert_eq!(hidden.len(), oracle.source_dimension());
    assert!(hidden.iter().any(|&value| value != 0.0));

    let mut top = [(0u32, 0f32); 3];
    assert_eq!(oracle.top_k(3, &mut top), 3);
    assert!(top[0].1 >= top[1].1 && top[1].1 >= top[2].1);
    for &(token, prob) in &top {
        assert!((token as usize) < oracle.vocab());
        assert!(prob > 0.0 && prob <= 1.0);
    }
    // Canonical ordering cross-checked against the spec recomputed
    // independently from this step's logits: probability descending, ties
    // broken by lower token id.
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0f32;
    let mut probs = vec![0f32; logits.len()];
    for (prob, &logit) in probs.iter_mut().zip(&logits) {
        *prob = (logit - max).exp();
        sum += *prob;
    }
    for prob in probs.iter_mut() {
        *prob /= sum;
    }
    let mut order: Vec<u32> = (0..logits.len() as u32).collect();
    order.sort_by(|a, b| {
        probs[*b as usize]
            .total_cmp(&probs[*a as usize])
            .then_with(|| a.cmp(b))
    });
    for (got, &want) in top.iter().zip(&order) {
        assert_eq!(got.0, want);
        assert_eq!(got.1, probs[want as usize]);
    }
    // Fewer output slots than k truncates to the slots available.
    let mut short = [(0u32, 0f32); 2];
    assert_eq!(oracle.top_k(3, &mut short), 2);
    assert_eq!(short[0], top[0]);
    assert_eq!(short[1], top[1]);
}

// ------------------------------------------------------------------ CLI --

/// #597 plumbing seam: `--source-manifest-kappa` parses into the observe
/// options, and the writer setter persists the κ into the observation
/// manifest atomically and idempotently across reopen.
#[test]
fn source_manifest_kappa_parses_and_persists_in_the_manifest() {
    let kappa = format!("blake3:{}", "7".repeat(64));
    let args: Vec<String> = [
        "--checkpoint",
        "/tmp/does-not-need-to-exist-for-parsing",
        "--source-manifest-kappa",
        kappa.as_str(),
    ]
    .map(str::to_owned)
    .to_vec();
    let options = uor_r4_graph_compiler::parse_observe_options(&args).expect("valid options");
    assert_eq!(
        options.source_manifest_kappa.as_deref(),
        Some(kappa.as_str())
    );
    // The compile parser accepts the same flag.
    let compile_args: Vec<String> = ["--source-manifest-kappa", kappa.as_str()]
        .map(str::to_owned)
        .to_vec();
    let compile_options =
        uor_r4_graph_compiler::parse_options(&compile_args).expect("valid options");
    assert_eq!(
        compile_options.source_manifest_kappa.as_deref(),
        Some(kappa.as_str())
    );

    let dir = unique_path("observe-source-kappa");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    assert_eq!(writer.manifest().source_manifest_kappa, None);
    writer
        .set_source_manifest_kappa(&kappa)
        .expect("set and store");
    // Idempotent: re-setting the recorded value does not rewrite.
    writer
        .set_source_manifest_kappa(&kappa)
        .expect("idempotent set");
    drop(writer);
    let manifest = ObservationManifest::load(&dir)
        .expect("manifest io")
        .expect("manifest present");
    assert_eq!(
        manifest.source_manifest_kappa.as_deref(),
        Some(kappa.as_str())
    );
    // A reopened writer (as `observe_sharded` does after the CLI pre-set)
    // preserves the stored κ.
    let reopened = ObservationShardWriter::open(&dir, 2).expect("reopen");
    assert_eq!(
        reopened.manifest().source_manifest_kappa.as_deref(),
        Some(kappa.as_str())
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// #600 plumbing seam: `--geometry-projection` parses a typed
/// [`GeometryProjection`] into the compile options, the writer setter
/// persists it into the observation manifest atomically and idempotently
/// across reopen, and an unset record leaves legacy manifest bytes
/// unchanged (no `geometry` key at all).
#[test]
fn geometry_projection_parses_and_persists_in_the_manifest() {
    let record = GeometryProjection::bucket_average(576, 288);
    let json = serde_json::to_string(&record).expect("record serializes");

    // The compile parser accepts the typed record as JSON.
    let compile_args: Vec<String> = ["--geometry-projection", json.as_str()]
        .map(str::to_owned)
        .to_vec();
    let compile_options =
        uor_r4_graph_compiler::parse_options(&compile_args).expect("valid options");
    assert_eq!(compile_options.geometry.as_ref(), Some(&record));
    // Malformed JSON is not a product of the arguments.
    let bad_args: Vec<String> = ["--geometry-projection", "{not json"]
        .map(str::to_owned)
        .to_vec();
    assert!(uor_r4_graph_compiler::parse_options(&bad_args).is_none());

    let dir = unique_path("observe-geometry");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    assert_eq!(writer.manifest().geometry, None);
    // Legacy bytes: an unset record serializes no `geometry` key.
    let legacy_json = serde_json::to_string(writer.manifest()).expect("manifest serializes");
    assert!(
        !legacy_json.contains("geometry"),
        "an unset record must leave legacy manifest bytes unchanged"
    );
    // And a legacy manifest without the key still deserializes (None).
    let legacy: ObservationManifest =
        serde_json::from_str(&legacy_json).expect("legacy manifest deserializes");
    assert_eq!(legacy.geometry, None);

    writer.set_geometry(&record).expect("set and store");
    // Idempotent: re-setting the recorded value does not rewrite.
    writer.set_geometry(&record).expect("idempotent set");
    drop(writer);
    let manifest = ObservationManifest::load(&dir)
        .expect("manifest io")
        .expect("manifest present");
    assert_eq!(manifest.geometry.as_ref(), Some(&record));
    // Round trip: the persisted record parses back bit-for-bit, digest
    // included.
    assert_eq!(
        manifest.geometry.as_ref().map(|g| &g.implementation_digest),
        Some(&record.implementation_digest)
    );
    // A reopened writer (as `observe_sharded` does after the pre-set)
    // preserves the stored record.
    let reopened = ObservationShardWriter::open(&dir, 2).expect("reopen");
    assert_eq!(reopened.manifest().geometry.as_ref(), Some(&record));
    let _ = std::fs::remove_dir_all(&dir);
}

/// #601 provenance seam: the writer setter persists the typed
/// [`TokenizerAdapter`] record into the observation manifest atomically
/// and idempotently across reopen, and an unset record leaves legacy
/// manifest bytes unchanged (no `tokenizer_adapter` key at all) — the
/// legacy llama2.c selection never sets one, so its historical bytes and
/// existing tokenizer CIDs stay valid.
#[test]
fn tokenizer_adapter_persists_in_the_manifest() {
    let tokenizer_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ab": 2}, "merges": ["a b"]}
    }"#;
    let record = uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer::from_tokenizer_json_bytes(
        tokenizer_json.as_bytes(),
    )
    .expect("fixture tokenizer parses")
    .adapter();

    let dir = unique_path("observe-tokenizer-adapter");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    assert_eq!(writer.manifest().tokenizer_adapter, None);
    // Legacy bytes: an unset record serializes no `tokenizer_adapter` key.
    let legacy_json = serde_json::to_string(writer.manifest()).expect("manifest serializes");
    assert!(
        !legacy_json.contains("tokenizer_adapter"),
        "an unset record must leave legacy manifest bytes unchanged"
    );
    // And a legacy manifest without the key still deserializes (None).
    let legacy: ObservationManifest =
        serde_json::from_str(&legacy_json).expect("legacy manifest deserializes");
    assert_eq!(legacy.tokenizer_adapter, None);

    // Other identity metadata and empty payload files do not constitute a
    // tokenizer era. This is the call order used by the text-observation
    // preparation path before its first record.
    writer
        .set_partition_rule("fixture-partition-rule")
        .expect("persist other identity metadata");
    std::fs::write(dir.join(shard_file_name(2, 0)), []).expect("empty shard placeholder");
    writer
        .set_tokenizer_adapter(&record)
        .expect("set and store");
    // Idempotent: re-setting the recorded value does not rewrite.
    writer
        .set_tokenizer_adapter(&record)
        .expect("idempotent set");
    drop(writer);
    let manifest = ObservationManifest::load(&dir)
        .expect("manifest io")
        .expect("manifest present");
    assert_eq!(manifest.tokenizer_adapter.as_ref(), Some(&record));
    // Round trip: the persisted record parses back bit-for-bit, digest
    // included.
    assert_eq!(
        manifest
            .tokenizer_adapter
            .as_ref()
            .map(|a| &a.adapter_digest),
        Some(&record.adapter_digest)
    );
    // A reopened writer (as the observe-text drivers do after the
    // pre-set) preserves the stored record.
    let reopened = ObservationShardWriter::open(&dir, 2).expect("reopen");
    assert_eq!(
        reopened.manifest().tokenizer_adapter.as_ref(),
        Some(&record)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The setter is also the admission boundary for public/custom callers: an
/// unregistered family/version is refused before a manifest can be created.
#[test]
fn tokenizer_adapter_unknown_version_is_refused_before_any_mutation() {
    let tokenizer_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ab": 2}, "merges": ["a b"]}
    }"#;
    let mut unknown = fixture_byte_bpe_adapter(tokenizer_json);
    unknown.version += 1;
    unknown.adapter_digest = unknown.declared_digest();

    let dir = unique_path("observe-tokenizer-adapter-unknown-version");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    let bytes_before = directory_bytes(&dir);
    let manifest_before = writer.manifest().clone();
    let error = writer
        .set_tokenizer_adapter(&unknown)
        .expect_err("unknown registry version must be refused");
    assert!(matches!(
        error.kind,
        uor_r4_model_source::SourceIngestKind::UnknownTokenizerAdapter {
            ref family,
            version,
        } if family == &unknown.family && version == unknown.version
    ));
    assert!(error.reason.contains("hf-byte-bpe/1"));
    assert!(error.reason.contains("sentencepiece-unigram/1"));
    assert!(error.reason.contains("sentencepiece-unigram/2"));
    assert_eq!(writer.manifest(), &manifest_before);
    assert_eq!(directory_bytes(&dir), bytes_before);

    let _ = std::fs::remove_dir_all(&dir);
}

/// A registered key is insufficient when the claimed adapter digest does not
/// reproduce from the canonical policy/CID fields. Both policy tampering and
/// direct digest tampering are refused without creating a manifest.
#[test]
fn tokenizer_adapter_inconsistent_digest_is_refused_before_any_mutation() {
    let tokenizer_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ab": 2}, "merges": ["a b"]}
    }"#;
    let valid = fixture_byte_bpe_adapter(tokenizer_json);
    let mut altered_policy = valid.clone();
    altered_policy.policy.normalizer = "tampered-normalizer".to_owned();
    let mut altered_digest = valid;
    altered_digest.adapter_digest = format!("blake3:{}", "0".repeat(64));

    let dir = unique_path("observe-tokenizer-adapter-inconsistent-digest");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    let bytes_before = directory_bytes(&dir);
    let manifest_before = writer.manifest().clone();
    for candidate in [&altered_policy, &altered_digest] {
        let error = writer
            .set_tokenizer_adapter(candidate)
            .expect_err("inconsistent adapter digest must be refused");
        assert!(
            error.reason.contains("canonical fields")
                && error.reason.contains("inconsistent provenance")
                && error.reason.contains("before mutation"),
            "focused digest diagnostic: {error}"
        );
        assert_eq!(writer.manifest(), &manifest_before);
        assert_eq!(directory_bytes(&dir), bytes_before);
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// Canonical address syntax is an independent admission rule. In particular,
/// recomputing the declared digest after putting an uppercase/non-address value
/// in `tokenizer_cid` must not make that internally self-consistent record
/// persistable.
#[test]
fn tokenizer_adapter_noncanonical_addresses_are_refused_before_any_mutation() {
    let tokenizer_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ab": 2}, "merges": ["a b"]}
    }"#;
    let valid = fixture_byte_bpe_adapter(tokenizer_json);

    let mut uppercase_cid = valid.clone();
    uppercase_cid.tokenizer_cid = format!("blake3:{}", "A".repeat(64));
    uppercase_cid.adapter_digest = uppercase_cid.declared_digest();
    let mut malformed_digest = valid;
    malformed_digest.adapter_digest = format!("blake3:{}", "B".repeat(64));

    let dir = unique_path("observe-tokenizer-adapter-noncanonical-address");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    let bytes_before = directory_bytes(&dir);
    let manifest_before = writer.manifest().clone();
    for (candidate, field) in [
        (&uppercase_cid, "tokenizer_cid"),
        (&malformed_digest, "adapter_digest"),
    ] {
        let error = writer
            .set_tokenizer_adapter(candidate)
            .expect_err("noncanonical address must be refused");
        assert!(
            error.reason.contains(field)
                && error.reason.contains("lowercase blake3:<64 hex>")
                && error.reason.contains("before mutation"),
            "focused canonical-address diagnostic: {error}"
        );
        assert_eq!(writer.manifest(), &manifest_before);
        assert_eq!(directory_bytes(&dir), bytes_before);
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// #639-4 incompatible-resume guard: an adapter identity is a one-time pin,
/// not mutable metadata. A different adapter must be rejected before the
/// manifest, completed shard records, or any other directory byte moves.
#[test]
fn tokenizer_adapter_mismatch_refuses_resume_before_any_mutation() {
    let first_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ab": 2}, "merges": ["a b"]}
    }"#;
    let second_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ba": 2}, "merges": ["b a"]}
    }"#;
    let first = uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer::from_tokenizer_json_bytes(
        first_json.as_bytes(),
    )
    .expect("first fixture tokenizer parses")
    .adapter();
    let second = uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer::from_tokenizer_json_bytes(
        second_json.as_bytes(),
    )
    .expect("second fixture tokenizer parses")
    .adapter();
    assert_ne!(first, second, "fixtures must declare distinct identities");

    let dir = unique_path("observe-tokenizer-adapter-mismatch");
    {
        let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
        writer
            .set_tokenizer_adapter(&first)
            .expect("pin first adapter");
        writer
            .write_record(&[0xA5; RECORD_SIZE], 0)
            .expect("write record");
        writer.finalize_all().expect("finalize observation corpus");
    }

    let bytes_before = directory_bytes(&dir);
    let records_before = merge_shards(&dir).expect("merge records before resume");
    let manifest_before = ObservationManifest::load(&dir)
        .expect("manifest io before resume")
        .expect("manifest before resume");
    let mut resumed = ObservationShardWriter::open(&dir, 2).expect("resume writer");

    // An identical reset is a byte-idempotent no-op.
    resumed
        .set_tokenizer_adapter(&first)
        .expect("identical adapter is idempotent");
    assert_eq!(directory_bytes(&dir), bytes_before);

    let error = resumed
        .set_tokenizer_adapter(&second)
        .expect_err("different adapter must refuse resume");
    assert!(
        error.reason.contains("incompatible resume")
            && error.reason.contains("refused before mutation")
            && error.reason.contains(&first.tokenizer_cid)
            && error.reason.contains(&second.tokenizer_cid),
        "focused mismatch diagnostic: {error}"
    );
    assert_eq!(resumed.manifest(), &manifest_before);
    assert_eq!(
        directory_bytes(&dir),
        bytes_before,
        "mismatch changed an observation-directory byte"
    );
    assert_eq!(
        merge_shards(&dir).expect("merge records after refused resume"),
        records_before,
        "mismatch changed completed observation records"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Removing the adapter field does not turn a completed corpus back into a
/// fresh one. The payload is historical evidence of its tokenizer era, so the
/// absent identity must be refused rather than retroactively relabelled.
#[test]
fn tokenizer_adapter_absence_cannot_relabel_completed_legacy_payload() {
    let tokenizer_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ab": 2}, "merges": ["a b"]}
    }"#;
    let adapter = uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer::from_tokenizer_json_bytes(
        tokenizer_json.as_bytes(),
    )
    .expect("fixture tokenizer parses")
    .adapter();
    let dir = unique_path("observe-tokenizer-adapter-legacy-completed");
    {
        let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
        writer
            .set_tokenizer_adapter(&adapter)
            .expect("pin adapter before payload");
        writer
            .write_record(&[0x5A; RECORD_SIZE], 0)
            .expect("write record");
        writer.finalize_all().expect("finalize observation corpus");
    }

    // Synthesize a historical manifest from before #601 by removing only the
    // optional adapter field from a completed corpus.
    let manifest_path = dir.join(MANIFEST_FILE);
    let mut legacy_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).expect("read manifest"))
            .expect("parse manifest");
    assert!(
        legacy_json
            .as_object_mut()
            .expect("manifest object")
            .remove("tokenizer_adapter")
            .is_some(),
        "fixture manifest must initially carry an adapter"
    );
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&legacy_json).expect("serialize legacy manifest"),
    )
    .expect("write legacy manifest");

    let bytes_before = directory_bytes(&dir);
    let records_before = merge_shards(&dir).expect("merge legacy records before resume");
    let manifest_before = ObservationManifest::load(&dir)
        .expect("manifest io before resume")
        .expect("legacy manifest before resume");
    assert_eq!(manifest_before.tokenizer_adapter, None);
    assert!(!manifest_before.completed.is_empty());

    let mut resumed = ObservationShardWriter::open(&dir, 2).expect("resume legacy writer");
    let error = resumed
        .set_tokenizer_adapter(&adapter)
        .expect_err("completed legacy payload cannot be relabelled");
    assert!(
        error.reason.contains("observation payload")
            && error.reason.contains("relabel legacy/unpinned")
            && error.reason.contains("before mutation"),
        "focused legacy-era diagnostic: {error}"
    );
    assert_eq!(resumed.manifest(), &manifest_before);
    assert_eq!(directory_bytes(&dir), bytes_before);
    assert_eq!(
        merge_shards(&dir).expect("merge legacy records after refusal"),
        records_before
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// A torn historical directory may retain only the tokenizer-dependent raw
/// observation checkpoint after its adapter field was stripped. `state.bin`
/// is still conclusive era evidence even when the manifest reports no records
/// or completed shards, so provenance pinning must fail without changing any
/// byte.
#[test]
fn tokenizer_adapter_absence_cannot_relabel_state_only_payload() {
    let tokenizer_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ab": 2}, "merges": ["a b"]}
    }"#;
    let adapter = fixture_byte_bpe_adapter(tokenizer_json);
    let dir = unique_path("observe-tokenizer-adapter-legacy-state-only");
    {
        let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
        writer
            .set_tokenizer_adapter(&adapter)
            .expect("persist adapter-bearing manifest");
    }

    let manifest_path = dir.join(MANIFEST_FILE);
    let mut legacy_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).expect("read manifest"))
            .expect("parse manifest");
    assert!(
        legacy_json
            .as_object_mut()
            .expect("manifest object")
            .remove("tokenizer_adapter")
            .is_some(),
        "fixture manifest must initially carry an adapter"
    );
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&legacy_json).expect("serialize stripped manifest"),
    )
    .expect("write stripped manifest");
    let state_path = dir.join(STATE_FILE);
    let state_bytes = [0xA5; 37];
    std::fs::write(&state_path, state_bytes).expect("write state-only checkpoint");

    let bytes_before = directory_bytes(&dir);
    let manifest_before = ObservationManifest::load(&dir)
        .expect("manifest io before resume")
        .expect("stripped manifest before resume");
    assert_eq!(manifest_before.tokenizer_adapter, None);
    assert_eq!(manifest_before.total_records, 0);
    assert!(manifest_before.completed.is_empty());

    let mut resumed = ObservationShardWriter::open(&dir, 2).expect("resume stripped writer");
    let error = resumed
        .set_tokenizer_adapter(&adapter)
        .expect_err("state-only legacy payload cannot be relabelled");
    assert!(
        error.reason.contains("observation payload")
            && error.reason.contains("relabel legacy/unpinned")
            && error.reason.contains("before mutation"),
        "focused state-only diagnostic: {error}"
    );
    assert_eq!(resumed.manifest(), &manifest_before);
    assert_eq!(directory_bytes(&dir), bytes_before);
    assert_eq!(
        std::fs::read(&state_path).expect("state survives refusal"),
        state_bytes
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// An interrupted writer can have payload bytes before any shard is complete.
/// Those bytes establish the legacy tokenizer era just as firmly as a
/// completed-manifest entry does.
#[test]
fn tokenizer_adapter_absence_cannot_relabel_partial_shard_payload() {
    let tokenizer_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ab": 2}, "merges": ["a b"]}
    }"#;
    let adapter = uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer::from_tokenizer_json_bytes(
        tokenizer_json.as_bytes(),
    )
    .expect("fixture tokenizer parses")
    .adapter();
    let dir = unique_path("observe-tokenizer-adapter-legacy-partial");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    writer
        .write_record(&[0x3C; RECORD_SIZE], 1)
        .expect("write partial record");
    writer.flush().expect("flush partial record");
    assert_eq!(writer.manifest().tokenizer_adapter, None);
    assert!(writer.manifest().completed.is_empty());

    let bytes_before = directory_bytes(&dir);
    let manifest_before = writer.manifest().clone();
    let error = writer
        .set_tokenizer_adapter(&adapter)
        .expect_err("partial legacy payload cannot be relabelled");
    assert!(error.reason.contains("observation payload"));
    assert_eq!(writer.manifest(), &manifest_before);
    assert_eq!(directory_bytes(&dir), bytes_before);

    let _ = std::fs::remove_dir_all(&dir);
}

/// Payload-path inspection is fail-closed: it examines the directory entry
/// itself rather than following a symlink (including a dangling one) or
/// accepting another non-regular object as an empty payload placeholder.
#[cfg(unix)]
#[test]
fn tokenizer_adapter_pin_refuses_non_regular_payload_path_before_mutation() {
    use std::os::unix::fs::symlink;

    let tokenizer_json = r#"{
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "model": {"type": "BPE", "vocab": {"a": 0, "b": 1, "ab": 2}, "merges": ["a b"]}
    }"#;
    let adapter = fixture_byte_bpe_adapter(tokenizer_json);
    let dir = unique_path("observe-tokenizer-adapter-payload-symlink");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    let payload_path = dir.join(shard_file_name(2, 0));
    let link_target = std::path::Path::new("missing-payload-target");
    symlink(link_target, &payload_path).expect("create dangling payload symlink");
    let entries_before: BTreeSet<_> = std::fs::read_dir(&dir)
        .expect("read directory before refusal")
        .map(|entry| entry.expect("directory entry").file_name())
        .collect();
    let manifest_before = writer.manifest().clone();

    let error = writer
        .set_tokenizer_adapter(&adapter)
        .expect_err("non-regular payload path must be refused");
    assert!(
        error.reason.contains("not a regular file")
            && error.reason.contains(&payload_path.display().to_string()),
        "focused payload-path diagnostic: {error}"
    );
    assert_eq!(writer.manifest(), &manifest_before);
    assert_eq!(
        std::fs::read_link(&payload_path).expect("symlink survives refusal"),
        link_target
    );
    let entries_after: BTreeSet<_> = std::fs::read_dir(&dir)
        .expect("read directory after refusal")
        .map(|entry| entry.expect("directory entry").file_name())
        .collect();
    assert_eq!(entries_after, entries_before);
    assert!(std::fs::symlink_metadata(dir.join(MANIFEST_FILE)).is_err());

    let _ = std::fs::remove_dir_all(&dir);
}

/// #602 provenance seam: the writer setter persists the typed
/// [`AttentionOperatorSpec`] record into the observation manifest
/// atomically and idempotently across reopen, and an unset record leaves
/// legacy manifest bytes unchanged (no `attention_operator` key at all)
/// — oracles that predate the record never set one, so their historical
/// bytes stay valid (the documented legacy interpretation).
#[test]
fn attention_operator_persists_in_the_manifest() {
    use uor_r4_model_source::attention::AttentionOperatorSpec;
    let record = AttentionOperatorSpec::standard();

    let dir = unique_path("observe-attention-operator");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    assert_eq!(writer.manifest().attention_operator, None);
    // Legacy bytes: an unset record serializes no `attention_operator` key.
    let legacy_json = serde_json::to_string(writer.manifest()).expect("manifest serializes");
    assert!(
        !legacy_json.contains("attention_operator"),
        "an unset record must leave legacy manifest bytes unchanged"
    );
    // And a legacy manifest without the key still deserializes (None).
    let legacy: ObservationManifest =
        serde_json::from_str(&legacy_json).expect("legacy manifest deserializes");
    assert_eq!(legacy.attention_operator, None);

    writer
        .set_attention_operator(&record)
        .expect("set and store");
    // Idempotent: re-setting the recorded value does not rewrite.
    writer
        .set_attention_operator(&record)
        .expect("idempotent set");
    drop(writer);
    let manifest = ObservationManifest::load(&dir)
        .expect("manifest io")
        .expect("manifest present");
    assert_eq!(manifest.attention_operator.as_ref(), Some(&record));
    // Round trip: the persisted record parses back bit-for-bit, digest
    // included.
    assert_eq!(
        manifest
            .attention_operator
            .as_ref()
            .map(|operator| &operator.implementation_digest),
        Some(&record.implementation_digest)
    );
    // A reopened writer (as the observe drivers do after the pre-set)
    // preserves the stored record.
    let reopened = ObservationShardWriter::open(&dir, 2).expect("reopen");
    assert_eq!(
        reopened.manifest().attention_operator.as_ref(),
        Some(&record)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// #600: the legacy checkpoint oracle declares no projection (its dim IS
/// the source width), while the Hugging Face adapter declares
/// `bucket-average/1` — asserted here structurally via the trait default.
/// #602 mirrors the pattern for the attention operator: an oracle that
/// declares nothing carries the documented legacy interpretation.
#[test]
fn trait_geometry_projection_defaults_to_none() {
    struct Plain;
    impl RepresentationSource for Plain {
        fn vocab_size(&self) -> usize {
            4
        }
        fn source_dimension(&self) -> usize {
            8
        }
        fn tokenizer_address(&self) -> &str {
            "plain"
        }
        fn read_embedding_rows(
            &self,
            _range: std::ops::Range<usize>,
            _output: &mut [f32],
        ) -> Option<()> {
            Some(())
        }
    }
    impl BehaviorSource for Plain {
        fn reset(&mut self) {}
        fn step(&mut self, _token: usize, _pos: usize, _logits: &mut [f32]) {}
    }
    impl TeacherOracle for Plain {
        fn vocab(&self) -> usize {
            4
        }
        fn dim(&self) -> usize {
            8
        }
        fn seq_len(&self) -> usize {
            8
        }
        fn kappa(&self) -> String {
            "blake3:plain".to_owned()
        }
        fn source_bytes(&self) -> usize {
            0
        }
        fn embedding(&self, _token: usize, _out: &mut [f32]) {}
    }
    assert_eq!(Plain.geometry_projection(), None);
    assert_eq!(Plain.attention_operator_spec(), None);
}

#[test]
fn observe_cli_writes_shards_and_resumes_without_rewriting() {
    if std::fs::metadata(LEGACY_CHECKPOINT).is_err() {
        eprintln!("skipping: source checkpoint not found at {LEGACY_CHECKPOINT}");
        return;
    }
    let dir = unique_path("observe-cli");
    let args: Vec<String> = [
        "observe",
        "--checkpoint",
        LEGACY_CHECKPOINT,
        "--seconds",
        "1",
        "--target",
        "1",
        "--shards",
        "3",
        "--out",
        dir.to_str().expect("utf-8 temp path"),
    ]
    .iter()
    .map(|arg| (*arg).to_string())
    .collect();
    uor_r4_graph_compiler::observe(&args[1..]).expect("observe run 1");

    let manifest = ObservationManifest::load(&dir)
        .expect("manifest io")
        .expect("manifest present");
    assert_eq!(manifest.shard_bits, 3);
    assert_eq!(manifest.completed.len(), 8);
    assert_eq!(manifest.total_records, 1);
    let mut mtimes = Vec::new();
    for shard in 0..8u32 {
        let path = dir.join(shard_file_name(3, shard));
        let metadata = std::fs::metadata(&path).expect("shard file");
        assert_eq!(metadata.len() % RECORD_SIZE as u64, 0);
        mtimes.push(metadata.modified().expect("mtime"));
    }
    let merged1 = merge_shards(&dir).expect("merge 1");
    assert_eq!(merged1.len(), RECORD_SIZE);

    // Rerun: every shard is complete, so nothing may be rewritten.
    uor_r4_graph_compiler::observe(&args[1..]).expect("observe run 2");
    let manifest2 = ObservationManifest::load(&dir)
        .expect("manifest io")
        .expect("manifest present");
    assert_eq!(manifest, manifest2, "manifest changed on resume");
    for (shard, mtime) in mtimes.iter().enumerate() {
        let path = dir.join(shard_file_name(3, shard as u32));
        assert_eq!(
            &std::fs::metadata(&path)
                .expect("shard file")
                .modified()
                .expect("mtime"),
            mtime,
            "completed shard {shard} rewritten on resume"
        );
    }
    assert_eq!(merge_shards(&dir).expect("merge 2"), merged1);

    let _ = std::fs::remove_dir_all(&dir);
}

// ----------------------------------------------- teacher trace (#603) --

/// #603 provenance seam: the writer setter persists the typed
/// [`TraceProfile`] record into the observation manifest atomically and
/// idempotently across reopen, and an unset record leaves legacy
/// manifest bytes unchanged (no `trace_profile` / `trace_row_bytes`
/// keys at all) — minimal passes never set one, so their historical
/// bytes stay valid (absence marks the minimal profile).
#[test]
fn trace_profile_persists_in_the_manifest() {
    let record = TraceProfile::layer(&[0, 2]);

    let dir = unique_path("observe-trace-profile");
    let mut writer = ObservationShardWriter::open(&dir, 2).expect("writer");
    assert_eq!(writer.manifest().trace_profile, None);
    // Legacy bytes: an unset record serializes no trace keys.
    let legacy_json = serde_json::to_string(writer.manifest()).expect("manifest serializes");
    assert!(
        !legacy_json.contains("trace_profile") && !legacy_json.contains("trace_row_bytes"),
        "an unset record must leave legacy manifest bytes unchanged"
    );
    // And a legacy manifest without the keys still deserializes (None).
    let legacy: ObservationManifest =
        serde_json::from_str(&legacy_json).expect("legacy manifest deserializes");
    assert_eq!(legacy.trace_profile, None);
    assert_eq!(legacy.trace_row_bytes, None);

    writer.set_trace_profile(&record).expect("set and store");
    // Idempotent: re-setting the recorded value does not rewrite.
    writer.set_trace_profile(&record).expect("idempotent set");
    drop(writer);
    let manifest = ObservationManifest::load(&dir)
        .expect("manifest io")
        .expect("manifest present");
    assert_eq!(manifest.trace_profile.as_ref(), Some(&record));
    // Round trip: the persisted record parses back bit-for-bit, digest
    // included.
    assert_eq!(
        manifest
            .trace_profile
            .as_ref()
            .map(|profile| &profile.declared_digest),
        Some(&record.declared_digest)
    );
    // A reopened writer preserves the stored record.
    let reopened = ObservationShardWriter::open(&dir, 2).expect("reopen");
    assert_eq!(reopened.manifest().trace_profile.as_ref(), Some(&record));
    let _ = std::fs::remove_dir_all(&dir);
}

/// #603 bundle identity: `identity_bundle_digest` moves when ANY of the
/// six components changes, is independent of the order the components
/// were recorded in, and distinguishes an ABSENT component from an
/// empty one (absence is an explicit marker in the digest input, never
/// a zero/default value).
#[test]
fn identity_bundle_digest_covers_components_order_and_absence() {
    use uor_r4_model_source::attention::AttentionOperatorSpec;

    let base = ObservationManifest::new(2);
    let mut digests = vec![base.identity_bundle_digest()];

    let mut with_input = base.clone();
    with_input.input_cid = Some(format!("blake3:{}", "1".repeat(64)));
    digests.push(with_input.identity_bundle_digest());

    let mut with_source = base.clone();
    with_source.source_manifest_kappa = Some(format!("blake3:{}", "2".repeat(64)));
    digests.push(with_source.identity_bundle_digest());

    let mut with_geometry = base.clone();
    with_geometry.geometry = Some(GeometryProjection::bucket_average(576, 288));
    digests.push(with_geometry.identity_bundle_digest());

    let mut with_operator = base.clone();
    with_operator.attention_operator = Some(AttentionOperatorSpec::standard());
    digests.push(with_operator.identity_bundle_digest());

    let mut with_profile = base.clone();
    with_profile.trace_profile = Some(TraceProfile::layer(&[0]));
    digests.push(with_profile.identity_bundle_digest());

    // Every single-component change is a distinct bundle identity.
    for (i, a) in digests.iter().enumerate() {
        for (j, b) in digests.iter().enumerate().skip(i + 1) {
            assert_ne!(a, b, "digests {i} and {j} collide");
        }
    }
    // A component VALUE change moves the digest too.
    let mut with_other_profile = base.clone();
    with_other_profile.trace_profile = Some(TraceProfile::layer(&[1]));
    assert_ne!(
        with_profile.identity_bundle_digest(),
        with_other_profile.identity_bundle_digest()
    );

    // Absent is NOT empty: an empty-string κ is a different identity
    // from an unset one.
    let mut with_empty = base.clone();
    with_empty.source_manifest_kappa = Some(String::new());
    assert_ne!(
        base.identity_bundle_digest(),
        with_empty.identity_bundle_digest()
    );
    assert_ne!(
        with_source.identity_bundle_digest(),
        with_empty.identity_bundle_digest()
    );

    // Set-order independence, through the persisted writer path.
    let geometry = GeometryProjection::bucket_average(576, 288);
    let cid = format!("blake3:{}", "3".repeat(64));
    let dir_a = unique_path("bundle-order-a");
    let mut writer = ObservationShardWriter::open(&dir_a, 2).expect("writer a");
    writer.set_input_cid(&cid).expect("cid first");
    writer.set_geometry(&geometry).expect("geometry second");
    drop(writer);
    let dir_b = unique_path("bundle-order-b");
    let mut writer = ObservationShardWriter::open(&dir_b, 2).expect("writer b");
    writer.set_geometry(&geometry).expect("geometry first");
    writer.set_input_cid(&cid).expect("cid second");
    drop(writer);
    let manifest_a = ObservationManifest::load(&dir_a)
        .expect("io a")
        .expect("manifest a");
    let manifest_b = ObservationManifest::load(&dir_b)
        .expect("io b")
        .expect("manifest b");
    assert_eq!(
        manifest_a.identity_bundle_digest(),
        manifest_b.identity_bundle_digest(),
        "the bundle digest must not depend on field-set order"
    );
    for dir in [&dir_a, &dir_b] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// A deterministic capture-capable oracle: logits, hidden state, and
/// every trace lane are pure functions of (token, pos), so two runs
/// under the same profile must produce byte-identical shards and
/// sidecars.
struct FakeTraceOracle {
    hidden: Vec<f32>,
}

impl FakeTraceOracle {
    const LAYERS: usize = 2;
    const HEADS: usize = 2;
    const KV_HEADS: usize = 1;
    const WIDTH: usize = 4;

    fn new() -> Self {
        Self {
            hidden: vec![0.0; Self::WIDTH],
        }
    }

    fn fill_logits(token: usize, pos: usize, logits: &mut [f32]) {
        for (i, logit) in logits.iter_mut().enumerate() {
            *logit = ((token * 31 + pos * 17 + i * 7) % 23) as f32 / 4.0;
        }
    }
}

impl RepresentationSource for FakeTraceOracle {
    fn vocab_size(&self) -> usize {
        16
    }
    fn source_dimension(&self) -> usize {
        Self::WIDTH
    }
    fn tokenizer_address(&self) -> &str {
        "fake-trace-tokenizer"
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

impl BehaviorSource for FakeTraceOracle {
    fn reset(&mut self) {
        self.hidden.fill(0.0);
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        Self::fill_logits(token, pos, logits);
        for (j, value) in self.hidden.iter_mut().enumerate() {
            *value = (token * 3 + pos * 5 + j) as f32 / 8.0;
        }
    }
}

impl TeacherOracle for FakeTraceOracle {
    fn vocab(&self) -> usize {
        16
    }
    fn dim(&self) -> usize {
        Self::WIDTH
    }
    fn seq_len(&self) -> usize {
        6
    }
    fn kappa(&self) -> String {
        "blake3:fake-trace".to_string()
    }
    fn source_bytes(&self) -> usize {
        0
    }
    fn embedding(&self, _token: usize, out: &mut [f32]) {
        out.fill(0.0);
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.hidden)
    }
    fn trace_capture_geometry(&self) -> Option<TraceCaptureGeometry> {
        Some(TraceCaptureGeometry {
            layers: Self::LAYERS,
            heads: Self::HEADS,
            kv_heads: Self::KV_HEADS,
            residual_width: Self::WIDTH,
        })
    }
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) -> bool {
        self.step(token, pos, logits);
        let kv_width = Self::WIDTH * Self::KV_HEADS / Self::HEADS;
        for layer in 0..Self::LAYERS {
            if request.attention_layers.contains(&layer) {
                for head in 0..Self::HEADS {
                    let mut att: Vec<f32> = (0..=pos)
                        .map(|t| (t + head * 2 + layer * 3 + token % 5 + 1) as f32)
                        .collect();
                    let total: f32 = att.iter().sum();
                    att.iter_mut().for_each(|w| *w /= total);
                    (sinks.attention)(layer, head, &att);
                }
            }
            if request.qkv_layers.contains(&layer) {
                let q: Vec<f32> = (0..Self::WIDTH)
                    .map(|i| (layer * 7 + token + pos * 2 + i) as f32 / 3.0)
                    .collect();
                let k: Vec<f32> = (0..kv_width)
                    .map(|i| (layer * 11 + token * 2 + pos + i) as f32 / 5.0)
                    .collect();
                let v: Vec<f32> = (0..kv_width)
                    .map(|i| (layer * 13 + token + pos + i) as f32 / 7.0)
                    .collect();
                (sinks.qkv)(layer, &q, &k, &v);
            }
            if request.residual_layers.contains(&layer) {
                let residual: Vec<f32> = (0..Self::WIDTH)
                    .map(|i| (layer * 17 + token * 5 + pos * 3 + i) as f32 / 9.0)
                    .collect();
                (sinks.residual)(layer, &residual);
            }
        }
        true
    }
}

fn traced_run_bytes(dir: &std::path::Path, shard_bits: u8) -> Vec<Vec<u8>> {
    let manifest = ObservationManifest::load(dir)
        .expect("manifest io")
        .expect("manifest present");
    let mut files = Vec::new();
    for shard in 0..manifest.shard_count() {
        let name = shard_file_name(shard_bits, shard);
        files.push(std::fs::read(dir.join(&name)).expect("shard bytes"));
        files.push(std::fs::read(dir.join(format!("{name}.prob"))).expect("prob bytes"));
        let trace = dir.join(trace_sidecar_name(shard_bits, shard));
        files.push(if trace.exists() {
            std::fs::read(&trace).expect("trace bytes")
        } else {
            Vec::new()
        });
    }
    files
}

/// #603 determinism: the same inputs and profile produce byte-identical
/// shard, probability-sidecar, and trace-sidecar bytes across two
/// independent full runs; the trace sidecar is aligned (one fixed-width
/// row per record), registered per shard in the manifest with its own
/// κ, and merge order is canonical. A rerun over the complete corpus
/// rewrites nothing.
#[test]
fn traced_observation_is_deterministic_and_aligned() {
    let profile = TraceProfile::full(&[0, 1], 3);
    const SHARDS: u8 = 2;
    const TARGET: usize = 40;

    let run = |dir: &std::path::Path| {
        let mut oracle = FakeTraceOracle::new();
        let summary = observe_sharded_traced(&mut oracle, 30, TARGET, SHARDS, dir, None, &profile)
            .expect("traced run");
        assert!(summary.done, "target must be reached");
    };
    let dir_a = unique_path("traced-a");
    let dir_b = unique_path("traced-b");
    run(&dir_a);
    run(&dir_b);
    assert_eq!(
        traced_run_bytes(&dir_a, SHARDS),
        traced_run_bytes(&dir_b, SHARDS),
        "double run must be byte-identical"
    );

    let manifest = ObservationManifest::load(&dir_a)
        .expect("manifest io")
        .expect("manifest present");
    assert_eq!(manifest.trace_profile.as_ref(), Some(&profile));
    // Row width is the pure lane sum for this oracle's geometry:
    // residuals 2×4×4 + final hidden 4×4 + qkv 2×(4+2+2)×4 + support
    // 2 layers × 2 heads × 3 slots × 8.
    let expected_row = (2 * 4 * 4 + 4 * 4 + 2 * 8 * 4 + 2 * 2 * 3 * 8) as u64;
    assert_eq!(manifest.trace_row_bytes, Some(expected_row));
    for (shard, entry) in &manifest.completed {
        if entry.records == 0 {
            continue;
        }
        let trace_kappa = entry
            .trace_kappa
            .as_ref()
            .unwrap_or_else(|| panic!("shard {shard} trace κ"));
        let bytes =
            std::fs::read(dir_a.join(trace_sidecar_name(SHARDS, *shard))).expect("trace bytes");
        assert_eq!(bytes.len() as u64, entry.records * expected_row);
        assert_eq!(*trace_kappa, kappa_of(&bytes));
    }
    let merged_trace = merge_trace_rows(&dir_a).expect("merge trace");
    assert_eq!(
        merged_trace.len() as u64,
        manifest.total_records * expected_row
    );
    assert_eq!(
        merged_trace,
        merge_trace_rows(&dir_b).expect("merge trace b")
    );
    // Bounded support with explicit absence: position 0 has fewer prefix
    // positions than the cap, so marker slots must exist in the lane —
    // and they are the documented marker, not zeros.
    let marker = SUPPORT_ABSENT_MARKER.to_le_bytes();
    let marker_slot: Vec<u8> = [marker, marker].concat();
    assert!(
        merged_trace
            .windows(marker_slot.len())
            .any(|window| window == marker_slot),
        "unfilled support slots must carry the explicit absence marker"
    );

    // Rerun: complete corpus, nothing rewritten.
    let mut oracle = FakeTraceOracle::new();
    let summary = observe_sharded_traced(&mut oracle, 30, TARGET, SHARDS, &dir_a, None, &profile)
        .expect("resume over complete corpus");
    assert!(summary.done);
    assert_eq!(summary.written, 0);
    assert_eq!(
        traced_run_bytes(&dir_a, SHARDS),
        traced_run_bytes(&dir_b, SHARDS)
    );

    for dir in [&dir_a, &dir_b] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// #603 crash-safe resume for the trace sidecar, at the writer level
/// (extending the `shard_spill_manifest_resume_and_merge` discipline):
/// an interrupted pass whose writer is dropped without finalizing
/// resumes from the on-disk prefix and converges to the exact bytes of
/// a single-pass run — records, probability rows, and trace rows.
#[test]
fn trace_sidecar_resumes_from_partial_writes() {
    const TRACE_ROW: usize = 24;
    let records = synth_records();
    let probability = |i: usize| ProbabilityMetadata {
        target_logprob_nats: -0.25 - i as f32,
        entropy_bits: 1.0 + i as f32,
        top8_mass: 0.5,
        target_rank: i as u16,
    };
    let trace_row = |i: usize| -> Vec<u8> {
        (0..TRACE_ROW)
            .map(|j| ((i * 13 + j * 5) % 251) as u8)
            .collect()
    };
    let write_all = |writer: &mut ObservationShardWriter, range: std::ops::Range<usize>| {
        for i in range {
            assert!(
                writer
                    .write_record_with_probability_and_trace(
                        &records[i],
                        probability(i),
                        &trace_row(i),
                        record_shard(i),
                    )
                    .expect("traced write")
            );
        }
    };

    // Single pass: the reference bytes.
    let dir_single = unique_path("trace-resume-single");
    let mut writer = ObservationShardWriter::open(&dir_single, SHARD_BITS).expect("open single");
    write_all(&mut writer, 0..records.len());
    writer.finalize_all().expect("finalize single");
    let reference_trace = merge_trace_rows(&dir_single).expect("merge single");
    let reference_records = merge_shards(&dir_single).expect("merge records single");

    // Interrupted pass: half the records, writer dropped without
    // finalizing (the manifest keeps the pinned row width), then a
    // resumed writer appends the remainder and finalizes.
    let dir_resumed = unique_path("trace-resume-partial");
    {
        let mut writer =
            ObservationShardWriter::open(&dir_resumed, SHARD_BITS).expect("open partial");
        write_all(&mut writer, 0..records.len() / 2);
        writer.flush().expect("flush partial");
    }
    let mut writer = ObservationShardWriter::open(&dir_resumed, SHARD_BITS).expect("reopen");
    assert_eq!(
        writer.manifest().trace_row_bytes,
        Some(TRACE_ROW as u64),
        "the pinned row width survives the crash"
    );
    // A row of the wrong width is refused mid-corpus: the profile and
    // capture geometry are pinned by the manifest's row width.
    assert!(
        writer
            .write_record_with_probability_and_trace(
                &records[0],
                probability(0),
                &[0u8; TRACE_ROW + 8],
                record_shard(0),
            )
            .is_err()
    );
    write_all(&mut writer, records.len() / 2..records.len());
    writer.finalize_all().expect("finalize resumed");
    assert_eq!(
        merge_shards(&dir_resumed).expect("merge records resumed"),
        reference_records
    );
    assert_eq!(
        merge_trace_rows(&dir_resumed).expect("merge resumed"),
        reference_trace,
        "a resumed traced pass must converge to the single-pass bytes"
    );

    for dir in [&dir_single, &dir_resumed] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// #603 absence stays absence: richer profiles are REFUSED — never
/// zero-filled — when the oracle has no capture surface; a corpus is
/// pinned to one profile (minimal cannot become traced mid-corpus and
/// vice versa); and the default minimal path records no trace fields at
/// all, byte-identical to a pre-#603 pass.
#[test]
fn trace_profiles_are_refused_not_zero_filled_and_pinned_per_corpus() {
    // An oracle without the capture surface refuses richer profiles.
    let dir = unique_path("trace-refused");
    let mut plain = FakeOracle { dim: 4, vocab: 8 };
    let error =
        observe_sharded_traced(&mut plain, 5, 16, 2, &dir, None, &TraceProfile::layer(&[0]))
            .expect_err("no capture surface is a refusal, not zeros");
    assert!(
        error.reason.contains("capture"),
        "reason names the gap: {error}"
    );
    let _ = std::fs::remove_dir_all(&dir);

    // Minimal through the traced entry point IS the minimal path: same
    // bytes as observe_sharded, no trace fields in the manifest.
    let dir_minimal = unique_path("trace-minimal");
    let dir_plain = unique_path("trace-plain");
    let mut oracle = FakeTraceOracle::new();
    observe_sharded_traced(
        &mut oracle,
        30,
        24,
        2,
        &dir_minimal,
        None,
        &TraceProfile::minimal(),
    )
    .expect("minimal traced run");
    let mut oracle = FakeTraceOracle::new();
    observe_sharded(&mut oracle, 30, 24, 2, &dir_plain, None).expect("plain run");
    assert_eq!(
        traced_run_bytes(&dir_minimal, 2),
        traced_run_bytes(&dir_plain, 2),
        "the minimal profile is exactly today's bytes"
    );
    let manifest = ObservationManifest::load(&dir_minimal)
        .expect("io")
        .expect("manifest");
    assert_eq!(manifest.trace_profile, None);
    assert_eq!(manifest.trace_row_bytes, None);

    // Profile pinning: a minimal corpus refuses a traced resume, and a
    // traced corpus refuses a minimal (or different-profile) resume.
    let mut oracle = FakeTraceOracle::new();
    assert!(
        observe_sharded_traced(
            &mut oracle,
            30,
            24,
            2,
            &dir_plain,
            None,
            &TraceProfile::layer(&[0]),
        )
        .is_err(),
        "a profile cannot be introduced mid-corpus"
    );
    let dir_traced = unique_path("trace-pinned");
    let mut oracle = FakeTraceOracle::new();
    observe_sharded_traced(
        &mut oracle,
        30,
        24,
        2,
        &dir_traced,
        None,
        &TraceProfile::layer(&[0]),
    )
    .expect("traced run");
    let mut oracle = FakeTraceOracle::new();
    assert!(
        observe_sharded(&mut oracle, 30, 24, 2, &dir_traced, None).is_err(),
        "a traced corpus refuses a minimal resume"
    );
    let mut oracle = FakeTraceOracle::new();
    assert!(
        observe_sharded_traced(
            &mut oracle,
            30,
            24,
            2,
            &dir_traced,
            None,
            &TraceProfile::layer(&[1]),
        )
        .is_err(),
        "a traced corpus refuses a different profile"
    );

    for dir in [&dir_minimal, &dir_plain, &dir_traced] {
        let _ = std::fs::remove_dir_all(dir);
    }
}
