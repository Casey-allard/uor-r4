//! Observation pipeline v2 tests (graph-compiler plan §4.1 / §5 Phase 2):
//! content-addressed sample ids, deterministic shard partitioning, spill +
//! manifest + resume, ordered merge (T-invariance), the optional teacher
//! trace surface, and the `observe` CLI.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};
use uor_r4_graph_compiler::observation::{
    MANIFEST_FILE, ObservationManifest, ObservationSession, ObservationShardWriter,
    ProbabilityMetadata, RAW_COMMITTED_FILE, RECORD_SIZE, RawCommittedCheckpoint, RawResumeStatus,
    STATE_FILE, merge_probability_metadata, merge_shards, merge_trace_rows, message_bits_per_token,
    observe_sharded, observe_sharded_traced, preflight_raw_observation_in_session,
    recover_raw_observation_in_session, sample_id, shard_file_name, shard_of, trace_sidecar_name,
};
use uor_r4_graph_compiler::observation_text::preflight_text_observation_in_session;
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

// -------------------------------------- raw transaction checkpoint (#725) --

#[test]
fn raw_committed_checkpoint_is_canonical_and_strictly_decoded() {
    let manifest = ObservationManifest::new(1);
    let checkpoint =
        RawCommittedCheckpoint::new(&manifest, 3, 2, 0x0123_4567_89AB_CDEF, false, vec![2, 1])
            .expect("valid checkpoint");
    let encoded = checkpoint.encode().expect("canonical encoding");

    assert_eq!(&encoded[0..8], b"R4RAWCOM");
    assert_eq!(u16::from_le_bytes(encoded[8..10].try_into().unwrap()), 1);
    assert_eq!(encoded[10], 1, "shard_bits");
    assert_eq!(encoded[11], 0, "done");
    assert_eq!(
        u32::from_le_bytes(encoded[12..16].try_into().unwrap()),
        RECORD_SIZE as u32
    );
    assert_eq!(
        u32::from_le_bytes(encoded[16..20].try_into().unwrap()),
        16,
        "probability row width"
    );
    assert_eq!(encoded.len(), 88 + 2 * 8);
    assert_eq!(
        RawCommittedCheckpoint::decode(&encoded).expect("round trip"),
        checkpoint
    );
    assert_eq!(checkpoint.state(), (3, 2, 0x0123_4567_89AB_CDEF, false));
    assert_eq!(checkpoint.committed_records(), &[2, 1]);
    assert_eq!(checkpoint.shard_bits(), 1);
    assert_eq!(checkpoint.trace_row_bytes(), None);
    assert_eq!(
        checkpoint.encode().unwrap(),
        encoded,
        "timestamp-free bytes"
    );

    let rejects = |label: &str, bytes: Vec<u8>| {
        let error = match RawCommittedCheckpoint::decode(&bytes) {
            Ok(_) => panic!("{label} must be rejected"),
            Err(error) => error,
        };
        assert!(
            error.reason.contains("raw observation checkpoint")
                || error.reason.contains("raw checkpoint"),
            "{label}: focused diagnostic: {error}"
        );
    };

    rejects("truncated", encoded[..encoded.len() - 1].to_vec());
    let mut extra = encoded.clone();
    extra.push(0);
    rejects("extra byte", extra);
    let mut bad_magic = encoded.clone();
    bad_magic[0] ^= 0xFF;
    rejects("bad magic", bad_magic);
    let mut unknown_version = encoded.clone();
    unknown_version[8..10].copy_from_slice(&2u16.to_le_bytes());
    rejects("unknown version", unknown_version);
    let mut invalid_done = encoded.clone();
    invalid_done[11] = 2;
    rejects("invalid done", invalid_done);
    let mut wrong_record_width = encoded.clone();
    wrong_record_width[12..16].copy_from_slice(&87u32.to_le_bytes());
    rejects("wrong record width", wrong_record_width);
    let mut wrong_probability_width = encoded.clone();
    wrong_probability_width[16..20].copy_from_slice(&15u32.to_le_bytes());
    rejects("wrong probability width", wrong_probability_width);
    let mut fanout_mismatch = encoded.clone();
    fanout_mismatch[84..88].copy_from_slice(&1u32.to_le_bytes());
    rejects("fan-out mismatch", fanout_mismatch);
    let mut total_mismatch = encoded.clone();
    total_mismatch[28..36].copy_from_slice(&4u64.to_le_bytes());
    rejects("record total mismatch", total_mismatch);

    assert!(
        RawCommittedCheckpoint::new(
            &ObservationManifest::new(0),
            u64::MAX,
            0,
            0,
            false,
            vec![u64::MAX],
        )
        .is_err(),
        "derived base/probability byte lengths must not overflow"
    );
    assert!(
        RawCommittedCheckpoint::new(
            &ObservationManifest::new(0),
            0,
            u64::from(u32::MAX) + 1,
            0,
            false,
            vec![0],
        )
        .is_err(),
        "story ids are a u32 wire field"
    );

    assert_eq!(RAW_COMMITTED_FILE, "raw-committed.bin");
}

fn raw_state_bytes(checkpoint: &RawCommittedCheckpoint) -> Vec<u8> {
    let (records, stories, rng, done) = checkpoint.state();
    let mut bytes = Vec::with_capacity(25);
    bytes.extend_from_slice(&records.to_le_bytes());
    bytes.extend_from_slice(&stories.to_le_bytes());
    bytes.extend_from_slice(&rng.to_le_bytes());
    bytes.push(u8::from(done));
    bytes
}

fn fixture_probability(rank: u16) -> ProbabilityMetadata {
    ProbabilityMetadata {
        target_logprob_nats: -0.25 - f32::from(rank),
        entropy_bits: 1.5 + f32::from(rank),
        top8_mass: 0.75,
        target_rank: rank,
    }
}

#[test]
fn raw_recovery_validates_every_shard_before_truncating_any_tail() {
    let dir = unique_path("raw-two-phase-late-invalid");
    let mut writer = ObservationShardWriter::open(&dir, 1).expect("writer");
    writer
        .write_record_with_probability(&[0x10; RECORD_SIZE], fixture_probability(0), 0)
        .expect("committed shard 0 row");
    writer
        .write_record_with_probability(&[0x20; RECORD_SIZE], fixture_probability(1), 1)
        .expect("committed shard 1 row");
    writer.flush().expect("flush committed prefix");
    let checkpoint =
        RawCommittedCheckpoint::new(writer.manifest(), 2, 1, 0xCAFE, false, vec![1, 1])
            .expect("checkpoint");

    // Earlier shard has a complete tentative row that would be truncated.
    writer
        .write_record_with_probability(&[0x30; RECORD_SIZE], fixture_probability(2), 0)
        .expect("tentative early row");
    writer.flush().expect("flush tentative row");
    drop(writer);
    std::fs::write(
        dir.join(RAW_COMMITTED_FILE),
        checkpoint.encode().expect("encode checkpoint"),
    )
    .expect("write checkpoint");
    std::fs::write(dir.join(STATE_FILE), raw_state_bytes(&checkpoint)).expect("write mirror");

    // A later companion has lost committed data. Recovery must discover this
    // in the read-only phase and leave the earlier tentative tail untouched.
    let late_probability = dir.join(format!("{}.prob", shard_file_name(1, 1)));
    std::fs::OpenOptions::new()
        .write(true)
        .open(&late_probability)
        .expect("open late companion")
        .set_len(0)
        .expect("shorten below checkpoint");
    let before = directory_bytes(&dir);

    let session = ObservationSession::acquire(&dir, 1).expect("session");
    let error = recover_raw_observation_in_session(&session)
        .expect_err("late short companion must fail the read-only phase");
    assert!(
        error.reason.contains("probability sidecar")
            && error.reason.contains("shorter than the committed"),
        "focused late-shard error: {error}"
    );
    drop(session);
    assert_eq!(
        directory_bytes(&dir),
        before,
        "a later invalid shard must precede every truncation and mirror change"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn raw_recovery_truncates_independent_tails_repairs_mirror_and_is_idempotent() {
    let dir = unique_path("raw-recovery-idempotent");
    let mut writer = ObservationShardWriter::open(&dir, 1).expect("writer");
    writer
        .write_record_with_probability(&[0x41; RECORD_SIZE], fixture_probability(0), 0)
        .expect("committed shard 0 row");
    writer
        .write_record_with_probability(&[0x42; RECORD_SIZE], fixture_probability(1), 1)
        .expect("committed shard 1 row");
    writer.flush().expect("flush committed prefix");
    let checkpoint =
        RawCommittedCheckpoint::new(writer.manifest(), 2, 7, 0x1234, false, vec![1, 1])
            .expect("checkpoint");
    drop(writer);
    std::fs::write(
        dir.join(RAW_COMMITTED_FILE),
        checkpoint.encode().expect("encode checkpoint"),
    )
    .expect("write checkpoint");

    // Different streams retain different supported aligned crash tails.
    let base0 = dir.join(shard_file_name(1, 0));
    let mut base0_bytes = std::fs::read(&base0).expect("base 0 prefix");
    base0_bytes.extend_from_slice(&[0xA0; RECORD_SIZE]);
    std::fs::write(&base0, base0_bytes).expect("append base-only tail");
    let probability1 = dir.join(format!("{}.prob", shard_file_name(1, 1)));
    let mut probability1_bytes = std::fs::read(&probability1).expect("probability 1 prefix");
    probability1_bytes.extend_from_slice(&fixture_probability(9).encode());
    std::fs::write(&probability1, probability1_bytes).expect("append probability-only tail");
    std::fs::write(dir.join(STATE_FILE), [0xFF; 9]).expect("write corrupt mirror");

    let session = ObservationSession::acquire(&dir, 1).expect("session");
    assert!(matches!(
        preflight_raw_observation_in_session(&session).expect("read-only preflight"),
        RawResumeStatus::Authoritative(ref observed) if observed == &checkpoint
    ));
    assert!(matches!(
        recover_raw_observation_in_session(&session).expect("recover"),
        RawResumeStatus::Authoritative(ref observed) if observed == &checkpoint
    ));
    drop(session);

    assert_eq!(std::fs::metadata(&base0).unwrap().len(), RECORD_SIZE as u64);
    assert_eq!(
        std::fs::metadata(&probability1).unwrap().len(),
        16,
        "probability companion returns to one committed row"
    );
    assert_eq!(
        std::fs::read(dir.join(STATE_FILE)).expect("repaired mirror"),
        raw_state_bytes(&checkpoint)
    );
    let once = directory_bytes(&dir);
    let session = ObservationSession::acquire(&dir, 1).expect("second session");
    recover_raw_observation_in_session(&session).expect("idempotent second recovery");
    drop(session);
    assert_eq!(directory_bytes(&dir), once, "second recovery rewrote bytes");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn raw_checkpoint_binds_identity_fanout_and_trace_layout_before_mutation() {
    let canonical_manifest = ObservationManifest::new(1);
    let canonical =
        RawCommittedCheckpoint::new(&canonical_manifest, 0, 0, 0x5EED, false, vec![0, 0])
            .expect("canonical checkpoint")
            .encode()
            .expect("encode checkpoint");

    let mut wrong_identity = canonical.clone();
    wrong_identity[52] ^= 1;
    let mut wrong_trace_width = canonical.clone();
    wrong_trace_width[20..28].copy_from_slice(&8u64.to_le_bytes());
    let wrong_fanout =
        RawCommittedCheckpoint::new(&ObservationManifest::new(0), 0, 0, 0x5EED, false, vec![0])
            .expect("other fan-out checkpoint")
            .encode()
            .expect("encode other fan-out");

    for (label, checkpoint_bytes, expected) in [
        ("identity", wrong_identity, "identity"),
        ("trace-layout", wrong_trace_width, "trace row width"),
        ("fan-out", wrong_fanout, "fan-out"),
    ] {
        let dir = unique_path(&format!("raw-bind-{label}"));
        std::fs::create_dir_all(&dir).expect("create directory");
        std::fs::write(dir.join(RAW_COMMITTED_FILE), checkpoint_bytes)
            .expect("write adversarial checkpoint");
        let before = directory_bytes(&dir);
        let session = ObservationSession::acquire(&dir, 1).expect("session");
        let error =
            preflight_raw_observation_in_session(&session).expect_err("layout mismatch must fail");
        assert!(error.reason.contains(expected), "{label}: {error}");
        drop(session);
        assert_eq!(directory_bytes(&dir), before, "{label} mutated bytes");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn raw_resume_refuses_mixed_text_checkpoint_formats_without_mutation() {
    for text_checkpoint in ["committed.bin", ".committed.bin.tmp"] {
        let dir = unique_path(&format!("raw-mixed-{text_checkpoint}"));
        std::fs::create_dir_all(&dir).expect("create directory");
        let checkpoint =
            RawCommittedCheckpoint::new(&ObservationManifest::new(0), 0, 0, 0x5EED, false, vec![0])
                .expect("raw checkpoint");
        std::fs::write(dir.join(RAW_COMMITTED_FILE), checkpoint.encode().unwrap())
            .expect("write raw checkpoint");
        std::fs::write(dir.join(text_checkpoint), b"text-checkpoint-evidence")
            .expect("write text checkpoint evidence");
        let before = directory_bytes(&dir);

        let session = ObservationSession::acquire(&dir, 0).expect("session");
        let error = preflight_raw_observation_in_session(&session)
            .expect_err("mixed driver formats must fail closed");
        assert!(
            error.reason.contains("text observation")
                && error.reason.contains("mixed raw observation resume"),
            "{text_checkpoint}: {error}"
        );
        drop(session);
        assert_eq!(directory_bytes(&dir), before);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(unix)]
#[test]
fn text_preflight_reciprocally_refuses_raw_checkpoint_entries_before_mutation() {
    use std::os::unix::fs::symlink;

    let input = unique_path("mixed-text-articles.jsonl");
    std::fs::write(
        &input,
        br#"{"id":"1","url":"https://example.invalid/1","title":"one","text":"ab"}
"#,
    )
    .expect("write article fixture");

    // A genuine regular raw checkpoint is valid directory content for the
    // common session layer, but belongs to the other driver and must be
    // rejected by text preflight before it publishes committed.bin/state.
    let regular = unique_path("text-refuses-regular-raw-checkpoint");
    let session = ObservationSession::acquire(&regular, 0).expect("session");
    std::fs::write(regular.join(RAW_COMMITTED_FILE), b"raw checkpoint evidence")
        .expect("write raw evidence");
    let before = directory_bytes(&regular);
    let error = preflight_text_observation_in_session(&session, &input, true, None, None, None)
        .expect_err("text driver must refuse a genuine raw checkpoint");
    assert!(
        error.reason.contains("raw observation")
            && error.reason.contains("mixed text observation resume"),
        "focused reciprocal mixed-format error: {error}"
    );
    assert_eq!(directory_bytes(&regular), before);
    drop(session);

    // Create adversarial entries after session acquisition so the public text
    // preflight itself exercises the common reload plus its explicit raw-entry
    // validation. Dangling links are inspected, never followed or ignored.
    let dangling = unique_path("text-refuses-dangling-raw-checkpoint");
    let session = ObservationSession::acquire(&dangling, 0).expect("session");
    let link_target = std::path::Path::new("missing-raw-checkpoint-target");
    symlink(link_target, dangling.join(RAW_COMMITTED_FILE)).expect("dangling checkpoint link");
    let error = preflight_text_observation_in_session(&session, &input, true, None, None, None)
        .expect_err("dangling raw checkpoint must fail text preflight");
    assert!(error.reason.contains("not a regular file"), "{error}");
    assert_eq!(
        std::fs::read_link(dangling.join(RAW_COMMITTED_FILE)).expect("link survives"),
        link_target
    );
    assert!(!dangling.join(STATE_FILE).exists());
    assert!(!dangling.join("committed.bin").exists());
    drop(session);

    let nonregular = unique_path("text-refuses-nonregular-raw-checkpoint");
    let session = ObservationSession::acquire(&nonregular, 0).expect("session");
    std::fs::create_dir(nonregular.join(RAW_COMMITTED_FILE))
        .expect("raw checkpoint directory entry");
    let error = preflight_text_observation_in_session(&session, &input, true, None, None, None)
        .expect_err("nonregular raw checkpoint must fail text preflight");
    assert!(error.reason.contains("not a regular file"), "{error}");
    assert!(nonregular.join(RAW_COMMITTED_FILE).is_dir());
    assert!(!nonregular.join(STATE_FILE).exists());
    assert!(!nonregular.join("committed.bin").exists());
    drop(session);

    for dir in [&regular, &dangling, &nonregular] {
        let _ = std::fs::remove_dir_all(dir);
    }
    let _ = std::fs::remove_file(&input);
}

#[cfg(unix)]
#[test]
fn raw_checkpoint_symlink_and_nonregular_entries_are_refused() {
    use std::os::unix::fs::symlink;

    let symlink_dir = unique_path("raw-checkpoint-symlink");
    std::fs::create_dir_all(&symlink_dir).expect("create symlink fixture");
    let victim = symlink_dir.with_extension("victim");
    std::fs::write(&victim, b"external checkpoint target").expect("write victim");
    symlink(&victim, symlink_dir.join(RAW_COMMITTED_FILE)).expect("checkpoint symlink");
    let error = ObservationSession::acquire(&symlink_dir, 0)
        .err()
        .expect("checkpoint symlink must be refused during session preflight");
    assert!(
        error.reason.contains("not a regular file"),
        "focused symlink refusal: {error}"
    );
    assert_eq!(
        std::fs::read(&victim).expect("victim survives"),
        b"external checkpoint target"
    );
    assert!(
        std::fs::symlink_metadata(symlink_dir.join(RAW_COMMITTED_FILE))
            .expect("link survives")
            .file_type()
            .is_symlink()
    );

    let directory_entry = unique_path("raw-checkpoint-directory-entry");
    std::fs::create_dir_all(directory_entry.join(RAW_COMMITTED_FILE))
        .expect("checkpoint directory entry");
    let error = ObservationSession::acquire(&directory_entry, 0)
        .err()
        .expect("checkpoint directory must be refused");
    assert!(error.reason.contains("not a regular file"), "{error}");

    let _ = std::fs::remove_dir_all(&symlink_dir);
    let _ = std::fs::remove_file(&victim);
    let _ = std::fs::remove_dir_all(&directory_entry);
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
fn fresh_raw_run_publishes_zero_checkpoint_before_any_append() {
    let dir = unique_path("raw-zero-checkpoint");
    std::fs::create_dir_all(&dir).expect("create output");
    let stale = dir.join(".raw-committed.bin.tmp-stale");
    std::fs::write(&stale, b"never authoritative").expect("stale temporary");

    let session = ObservationSession::acquire(&dir, 1).expect("fresh session");
    assert_eq!(
        preflight_raw_observation_in_session(&session).expect("stale temp is ignored"),
        RawResumeStatus::Fresh
    );
    drop(session);

    let mut oracle = FakeOracle { dim: 4, vocab: 8 };
    let summary =
        observe_sharded(&mut oracle, 0, 10, 1, &dir, None).expect("zero-budget initialization");
    assert_eq!(summary.records, 0);
    assert_eq!(summary.written, 0);
    assert!(!summary.done);

    let checkpoint = RawCommittedCheckpoint::decode(
        &std::fs::read(dir.join(RAW_COMMITTED_FILE)).expect("authoritative checkpoint"),
    )
    .expect("decode zero checkpoint");
    assert_eq!(checkpoint.state(), (0, 0, 0x5EED, false));
    assert_eq!(checkpoint.committed_records(), &[0, 0]);
    assert_eq!(
        std::fs::read(dir.join(STATE_FILE)).expect("state mirror"),
        raw_state_bytes(&checkpoint)
    );
    assert_eq!(
        std::fs::read(&stale).expect("stale temp remains inert"),
        b"never authoritative"
    );
    for shard in 0..2 {
        assert!(
            !dir.join(shard_file_name(1, shard)).exists(),
            "zero checkpoint must precede the first shard append"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// Small-fixture empirical record for #725. This intentionally has no timing
/// threshold: filesystem synchronization latency is host-dependent. Run with
/// `--ignored --nocapture` to report the canonical footprint plus median/mean
/// end-to-end publication latency for the authoritative zero boundary and its
/// `state.bin` mirror through the public observe entry point.
#[test]
#[ignore = "manual checkpoint publication overhead measurement"]
fn raw_checkpoint_small_fixture_overhead_measurement() {
    const ITERATIONS: usize = 31;
    const SHARD_BITS: u8 = 2;
    let dir = unique_path("raw-checkpoint-overhead");
    let mut samples_ns = Vec::with_capacity(ITERATIONS);

    // Warm the public path and establish the permanent out-of-directory lock
    // inode before sampling. Removal is outside the measured interval and is
    // safe because this is a private test directory with a zero checkpoint.
    let mut oracle = FakeOracle { dim: 4, vocab: 8 };
    observe_sharded(&mut oracle, 0, 1, SHARD_BITS, &dir, None).expect("warm-up publish");
    for _ in 0..ITERATIONS {
        std::fs::remove_file(dir.join(RAW_COMMITTED_FILE)).expect("remove prior raw checkpoint");
        std::fs::remove_file(dir.join(STATE_FILE)).expect("remove prior mirror");
        let mut oracle = FakeOracle { dim: 4, vocab: 8 };
        let started = std::time::Instant::now();
        observe_sharded(&mut oracle, 0, 1, SHARD_BITS, &dir, None)
            .expect("publish zero transaction");
        samples_ns.push(started.elapsed().as_nanos());
    }

    let checkpoint_bytes =
        std::fs::read(dir.join(RAW_COMMITTED_FILE)).expect("published checkpoint bytes");
    let checkpoint =
        RawCommittedCheckpoint::decode(&checkpoint_bytes).expect("decode measured checkpoint");
    assert_eq!(checkpoint.committed_records(), &[0, 0, 0, 0]);
    assert_eq!(checkpoint_bytes.len(), 88 + 8 * (1 << SHARD_BITS));
    assert_eq!(
        std::fs::metadata(dir.join(STATE_FILE)).unwrap().len(),
        25,
        "compatibility mirror footprint"
    );

    samples_ns.sort_unstable();
    let median_ns = samples_ns[ITERATIONS / 2];
    let mean_ns = samples_ns.iter().sum::<u128>() / ITERATIONS as u128;
    eprintln!(
        "raw_checkpoint_overhead: iterations={ITERATIONS} shards={} canonical_bytes={} mirror_bytes=25 median_publish_ns={median_ns} mean_publish_ns={mean_ns}",
        1usize << SHARD_BITS,
        checkpoint_bytes.len(),
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn missing_raw_checkpoint_accepts_only_fresh_or_finalized_legacy() {
    // Unfinished legacy bytes have no provable whole-story boundary.
    let incomplete = unique_path("raw-incomplete-legacy");
    let mut writer = ObservationShardWriter::open(&incomplete, 1).expect("legacy writer");
    writer
        .write_record_with_probability(&[0x71; RECORD_SIZE], fixture_probability(1), 0)
        .expect("legacy row");
    writer.flush().expect("flush legacy row");
    drop(writer);
    let mut unfinished_state = Vec::with_capacity(25);
    unfinished_state.extend_from_slice(&1u64.to_le_bytes());
    unfinished_state.extend_from_slice(&1u64.to_le_bytes());
    unfinished_state.extend_from_slice(&0xABCDu64.to_le_bytes());
    unfinished_state.push(0);
    std::fs::write(incomplete.join(STATE_FILE), unfinished_state).expect("legacy state");
    let before = directory_bytes(&incomplete);
    let mut oracle = FakeOracle { dim: 4, vocab: 8 };
    let error = observe_sharded(&mut oracle, 10, 2, 1, &incomplete, None)
        .expect_err("unfinished legacy must fail closed");
    assert!(
        error.reason.contains("unfinished legacy")
            && error.reason.contains("no authoritative raw-committed.bin")
            && error.reason.contains("fresh output directory"),
        "focused migration guidance: {error}"
    );
    assert_eq!(
        directory_bytes(&incomplete),
        before,
        "legacy refusal must precede mutation"
    );

    // A fully finalized legacy bundle has manifest κs and a done state that
    // prove every final boundary, so it remains readable without migration.
    let complete = unique_path("raw-complete-legacy");
    let mut oracle = FakeOracle { dim: 4, vocab: 8 };
    observe_sharded(&mut oracle, 10, 1, 1, &complete, None).expect("complete fixture");
    std::fs::remove_file(complete.join(RAW_COMMITTED_FILE))
        .expect("synthesize pre-checkpoint legacy bundle");
    let complete_before = directory_bytes(&complete);
    let mut oracle = FakeOracle { dim: 4, vocab: 8 };
    let summary = observe_sharded(&mut oracle, 10, 1, 1, &complete, None)
        .expect("finalized legacy remains replayable");
    assert!(summary.done);
    assert_eq!(summary.written, 0);
    assert_eq!(
        directory_bytes(&complete),
        complete_before,
        "finalized legacy bytes must not be rewritten or migrated"
    );

    for dir in [&incomplete, &complete] {
        let _ = std::fs::remove_dir_all(dir);
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

    // All identities are pinned before even an empty shard entry appears:
    // recognized payload presence fixes the legacy tokenizer era.
    writer
        .set_partition_rule("fixture-partition-rule")
        .expect("persist other identity metadata");
    writer
        .set_tokenizer_adapter(&record)
        .expect("set and store");
    std::fs::write(dir.join(shard_file_name(2, 0)), []).expect("empty shard placeholder");
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
    fn eos_token(&self) -> usize {
        // Keep the deterministic crash fixture at fixed six-row story
        // boundaries; sampled vocabulary tokens can never equal this value.
        usize::MAX
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

fn clone_observation_files(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("create cloned observation directory");
    for (name, bytes) in directory_bytes(from) {
        std::fs::write(to.join(name), bytes).expect("clone observation file");
    }
}

fn append_reference_rows(
    reference: &std::path::Path,
    resumed: &std::path::Path,
    name: &str,
    row_bytes: usize,
    committed_rows: usize,
    appended_rows: usize,
) {
    let reference_bytes = std::fs::read(reference.join(name)).unwrap_or_default();
    let mut resumed_bytes = std::fs::read(resumed.join(name)).unwrap_or_default();
    let committed_bytes = committed_rows
        .checked_mul(row_bytes)
        .expect("fixture committed byte length");
    assert_eq!(
        resumed_bytes.len(),
        committed_bytes,
        "{name}: prefix fixture agrees with checkpoint"
    );
    assert!(
        reference_bytes.len() >= committed_bytes + appended_rows * row_bytes,
        "{name}: reference contains requested tentative rows"
    );
    assert_eq!(
        resumed_bytes,
        reference_bytes[..committed_bytes],
        "{name}: committed prefix matches uninterrupted run"
    );
    resumed_bytes.extend_from_slice(
        &reference_bytes[committed_bytes..committed_bytes + appended_rows * row_bytes],
    );
    if !resumed_bytes.is_empty() {
        std::fs::write(resumed.join(name), resumed_bytes).expect("append tentative rows");
    }
}

/// Convert a finalized story-boundary fixture into the exact on-disk state of
/// a crash after heterogeneous aligned writes from later stories but before
/// the next authoritative checkpoint replacement.
fn craft_raw_crash_tails(
    prefix: &std::path::Path,
    reference: &std::path::Path,
    resumed: &std::path::Path,
) -> RawCommittedCheckpoint {
    clone_observation_files(prefix, resumed);
    let old_checkpoint = RawCommittedCheckpoint::decode(
        &std::fs::read(prefix.join(RAW_COMMITTED_FILE)).expect("prefix checkpoint"),
    )
    .expect("decode prefix checkpoint");
    let (records, stories, rng, _) = old_checkpoint.state();
    assert_eq!(records, 12, "fixture checkpoint is exactly two stories");
    assert_eq!(stories, 2, "fixed-width story fixture");

    let mut manifest = ObservationManifest::load(resumed)
        .expect("manifest io")
        .expect("prefix manifest");
    manifest.completed.clear();
    manifest.total_records = 0;
    std::fs::write(
        resumed.join(MANIFEST_FILE),
        serde_json::to_vec_pretty(&manifest).expect("serialize incomplete manifest"),
    )
    .expect("write incomplete manifest");
    let checkpoint = RawCommittedCheckpoint::new(
        &manifest,
        records,
        stories,
        rng,
        false,
        old_checkpoint.committed_records().to_vec(),
    )
    .expect("unfinished authoritative checkpoint");
    std::fs::write(
        resumed.join(RAW_COMMITTED_FILE),
        checkpoint.encode().expect("encode checkpoint"),
    )
    .expect("write checkpoint");
    std::fs::write(resumed.join(STATE_FILE), [0xD5; 25]).expect("stale state mirror");

    let reference_checkpoint = RawCommittedCheckpoint::decode(
        &std::fs::read(reference.join(RAW_COMMITTED_FILE)).expect("reference checkpoint"),
    )
    .expect("decode reference checkpoint");
    assert_eq!(reference_checkpoint.state().0, 24);
    let mut touched_shards = 0usize;
    let mut companion_skew = false;
    let mut trace_skew = false;
    for (shard, (&committed, &final_rows)) in checkpoint
        .committed_records()
        .iter()
        .zip(reference_checkpoint.committed_records())
        .enumerate()
    {
        let later_rows = usize::try_from(final_rows - committed).expect("fixture tail rows");
        if later_rows == 0 {
            continue;
        }
        touched_shards += 1;
        let base_name = shard_file_name(checkpoint.shard_bits(), shard as u32);
        append_reference_rows(
            reference,
            resumed,
            &base_name,
            RECORD_SIZE,
            committed as usize,
            later_rows,
        );
        let probability_rows = if shard.is_multiple_of(2) {
            0
        } else {
            later_rows.min(1)
        };
        append_reference_rows(
            reference,
            resumed,
            &format!("{base_name}.prob"),
            16,
            committed as usize,
            probability_rows,
        );
        companion_skew |= probability_rows != later_rows;
        if let Some(trace_row_bytes) = checkpoint.trace_row_bytes() {
            let trace_rows = if shard.is_multiple_of(3) {
                later_rows
            } else {
                later_rows.saturating_sub(1)
            };
            append_reference_rows(
                reference,
                resumed,
                &trace_sidecar_name(checkpoint.shard_bits(), shard as u32),
                trace_row_bytes as usize,
                committed as usize,
                trace_rows,
            );
            trace_skew |= trace_rows != later_rows || trace_rows != probability_rows;
        }
    }
    assert!(
        touched_shards >= 2,
        "deterministic fixture must exercise tentative tails on several shards"
    );
    assert!(companion_skew, "base/probability crash windows differ");
    if checkpoint.trace_row_bytes().is_some() {
        assert!(trace_skew, "base/probability/trace tails differ");
    }
    checkpoint
}

fn assert_raw_reference_resume_convergence(traced: bool) {
    const SHARD_BITS: u8 = 2;
    const PREFIX_RECORDS: usize = 12;
    const FINAL_RECORDS: usize = 24;
    let profile = TraceProfile::full(&[0, 1], 3);
    let reference = unique_path(if traced {
        "raw-reference-traced"
    } else {
        "raw-reference-minimal"
    });
    let prefix = unique_path(if traced {
        "raw-prefix-traced"
    } else {
        "raw-prefix-minimal"
    });
    let resumed = unique_path(if traced {
        "raw-resumed-traced"
    } else {
        "raw-resumed-minimal"
    });

    let run = |dir: &std::path::Path, target: usize| {
        let mut oracle = FakeTraceOracle::new();
        if traced {
            observe_sharded_traced(&mut oracle, 30, target, SHARD_BITS, dir, None, &profile)
                .expect("traced observation")
        } else {
            observe_sharded(&mut oracle, 30, target, SHARD_BITS, dir, None)
                .expect("minimal observation")
        }
    };
    assert!(run(&reference, FINAL_RECORDS).done);
    assert!(run(&prefix, PREFIX_RECORDS).done);
    let checkpoint = craft_raw_crash_tails(&prefix, &reference, &resumed);
    assert_eq!(checkpoint.state().0, PREFIX_RECORDS as u64);

    let summary = run(&resumed, FINAL_RECORDS);
    assert!(summary.done);
    assert_eq!(summary.records, FINAL_RECORDS as u64);
    assert_eq!(
        directory_bytes(&resumed),
        directory_bytes(&reference),
        "resume must converge checkpoint, mirror, manifest, shards, companions, and kappas byte-for-byte"
    );
    assert_eq!(
        merge_shards(&resumed).expect("resumed merged base"),
        merge_shards(&reference).expect("reference merged base")
    );
    assert_eq!(
        merge_probability_metadata(&resumed).expect("resumed merged probability"),
        merge_probability_metadata(&reference).expect("reference merged probability")
    );
    if traced {
        assert_eq!(
            merge_trace_rows(&resumed).expect("resumed merged trace"),
            merge_trace_rows(&reference).expect("reference merged trace")
        );
    }

    for dir in [&reference, &prefix, &resumed] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn minimal_raw_failure_injection_converges_to_uninterrupted_bytes() {
    assert_raw_reference_resume_convergence(false);
}

#[test]
fn traced_raw_failure_injection_converges_to_uninterrupted_bytes() {
    assert_raw_reference_resume_convergence(true);
}

#[test]
fn final_checkpoint_resume_is_idempotent_across_finalization_and_merge() {
    const SHARD_BITS: u8 = 2;
    const TARGET: usize = 12;
    let reference = unique_path("raw-finalization-reference");
    let resumed = unique_path("raw-finalization-resumed");
    let mut oracle = FakeTraceOracle::new();
    observe_sharded(&mut oracle, 30, TARGET, SHARD_BITS, &reference, None).expect("reference run");
    clone_observation_files(&reference, &resumed);

    let checkpoint = RawCommittedCheckpoint::decode(
        &std::fs::read(resumed.join(RAW_COMMITTED_FILE)).expect("final checkpoint"),
    )
    .expect("decode final checkpoint");
    assert_eq!(
        checkpoint.state(),
        (TARGET as u64, 2, checkpoint.state().2, true)
    );
    let mut manifest = ObservationManifest::load(&resumed)
        .expect("manifest io")
        .expect("manifest");
    manifest.completed.clear();
    manifest.total_records = 0;
    std::fs::write(
        resumed.join(MANIFEST_FILE),
        serde_json::to_vec_pretty(&manifest).expect("serialize incomplete manifest"),
    )
    .expect("simulate crash before finalization");
    std::fs::remove_file(resumed.join(STATE_FILE)).expect("simulate missing mirror");
    std::fs::write(resumed.join("merged.bin"), b"partial merged crash bytes")
        .expect("simulate interrupted merge");

    let mut oracle = FakeTraceOracle::new();
    let summary = observe_sharded(&mut oracle, 30, TARGET, SHARD_BITS, &resumed, None)
        .expect("resume final checkpoint");
    assert!(summary.done);
    assert_eq!(summary.written, 0);

    // This is the graph-cli done-path operation: an interrupted `merged.bin`
    // is derived afresh from the now κ-validated canonical shard order.
    for dir in [&reference, &resumed] {
        let merged = merge_shards(dir).expect("canonical merge");
        std::fs::write(dir.join("merged.bin"), merged).expect("publish merged fixture");
    }
    assert_eq!(
        directory_bytes(&resumed),
        directory_bytes(&reference),
        "checkpoint, finalization, mirror repair, and merge must converge"
    );

    for dir in [&reference, &resumed] {
        let _ = std::fs::remove_dir_all(dir);
    }
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
    writer
        .set_trace_profile(&TraceProfile::layer(&[0]))
        .expect("pin reference trace profile");
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
        writer
            .set_trace_profile(&TraceProfile::layer(&[0]))
            .expect("pin resumed trace profile");
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
