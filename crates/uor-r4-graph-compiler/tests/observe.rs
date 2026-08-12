//! Observation pipeline v2 tests (graph-compiler plan §4.1 / §5 Phase 2):
//! content-addressed sample ids, deterministic shard partitioning, spill +
//! manifest + resume, ordered merge (T-invariance), the optional teacher
//! trace surface, and the `observe` CLI.

use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};
use uor_r4_graph_compiler::observation::{
    ObservationManifest, ObservationShardWriter, ProbabilityMetadata, RECORD_SIZE,
    merge_probability_metadata, merge_shards, message_bits_per_token, sample_id, shard_file_name,
    shard_of,
};
use uor_r4_model_source::{BehaviorSource, LlamaOracle, RepresentationSource, TeacherOracle};

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
        "64",
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
    assert_eq!(manifest.total_records, 64);
    let mut mtimes = Vec::new();
    for shard in 0..8u32 {
        let path = dir.join(shard_file_name(3, shard));
        let metadata = std::fs::metadata(&path).expect("shard file");
        assert_eq!(metadata.len() % RECORD_SIZE as u64, 0);
        mtimes.push(metadata.modified().expect("mtime"));
    }
    let merged1 = merge_shards(&dir).expect("merge 1");
    assert_eq!(merged1.len(), 64 * RECORD_SIZE);

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
