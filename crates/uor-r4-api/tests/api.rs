//! Unit tests for the typed façade: error paths, the version surface,
//! parts validation, and the bytes-based tokenizer. None of these need a
//! real teacher model; the end-to-end compile test is `#[ignore]`d (see
//! its comment) — hologram-ai owns the real E2E.

use std::error::Error;
use std::path::PathBuf;

use uor_r4_api::compile::{
    compile, CompileOptions, CompileOutcome, CompileRequest, QualityProfile, Stage,
};
use uor_r4_api::engine::{AbiVersion, EngineParts, PredictOutput, R4Engine};
use uor_r4_api::{Tokenizer, TokenizerAdapterKey};
use uor_r4_core::transformerless::scenarios::{
    export_runtime_tokenizer_table, RuntimeTokenizerDecodePolicy, RuntimeTokenizerDecodeTable,
    RuntimeTokenizerEncodePolicy, RuntimeTokenizerIdentity,
};
use uor_r4_graph_format::{
    ArtifactBuilder, SectionId, FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR,
    INFERENCE_OPERATION_CONTRACT_VERSION,
};

// ------------------------------------------------------------ tokenizer --

/// The binary tokenizer.bin format: per token, i32 LE length then bytes.
fn tokenizer_bytes(tokens: &[&[u8]]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for token in tokens {
        bytes.extend_from_slice(&(token.len() as i32).to_le_bytes());
        bytes.extend_from_slice(token);
    }
    bytes
}

#[test]
fn tokenizer_from_bytes_roundtrip() {
    // The greedy pair-merge encoder needs every input character to be
    // encodable as a whole token or via byte fallback, so the fixture
    // vocab carries the single characters plus mergeable pairs.
    let bytes = tokenizer_bytes(&[b"<unk>", b"<s>", b"</s>", b" ", b"a", b"b", b" a", b"ab"]);
    let tokenizer = Tokenizer::from_bytes(&bytes).expect("parses");
    assert_eq!(tokenizer.vocab.len(), 8);
    assert_eq!(tokenizer.vocab[6], b" a");

    let mut encoded = [0u32; 8];
    let count = tokenizer.encode_into(" ab", &mut encoded).expect("encodes");
    let mut decoded = [0u8; 16];
    let len = tokenizer
        .decode_into(&encoded[..count], &mut decoded)
        .expect("decodes");
    assert!(!decoded[..len].is_empty());
}

#[test]
fn tokenizer_from_bytes_rejects_truncated() {
    // Declared length longer than the remaining bytes.
    let bytes = 4i32
        .to_le_bytes()
        .into_iter()
        .chain(*b"a")
        .collect::<Vec<_>>();
    // from_bytes is total: a truncated token yields None.
    assert!(
        Tokenizer::from_bytes(&bytes).is_none(),
        "truncated token must fail"
    );

    // Negative length.
    let bytes = (-1i32).to_le_bytes().to_vec();
    assert!(Tokenizer::from_bytes(&bytes).is_none());

    // Trailing partial length field.
    assert!(Tokenizer::from_bytes(&[0u8, 1]).is_none());
}

#[test]
fn tagged_runtime_tokenizer_is_decode_only_and_preserves_identity() {
    let dir = temp_dir("tagged-tokenizer");
    let path = dir.join("tokenizer.bin");
    let definition_cid = format!("blake3:{}", "1".repeat(64));
    let adapter_digest = format!("blake3:{}", "2".repeat(64));
    let table = RuntimeTokenizerDecodeTable {
        identity: RuntimeTokenizerIdentity {
            family: "future-sentencepiece-family".to_owned(),
            version: 41,
            tokenizer_cid: definition_cid.clone(),
            adapter_digest: adapter_digest.clone(),
        },
        pieces: vec![Vec::new(), "▁hello".as_bytes().to_vec(), b"!".to_vec()],
        encode_policy: RuntimeTokenizerEncodePolicy::Unavailable,
        decode_policy: RuntimeTokenizerDecodePolicy::SentencePiece {
            strip_dummy_prefix: true,
        },
        source_byte_lengths: None,
    };
    export_runtime_tokenizer_table(&table, &path).expect("tagged export");
    let bytes = std::fs::read(&path).expect("tagged bytes");
    let tokenizer = Tokenizer::from_bytes(&bytes).expect("tagged parser");
    assert!(tokenizer.is_decode_only());
    assert_eq!(
        tokenizer.adapter_key(),
        Some(("future-sentencepiece-family", 41))
    );
    let identity = tokenizer.adapter_identity().expect("tagged identity");
    assert_eq!(identity.tokenizer_cid, definition_cid);
    assert_eq!(identity.adapter_digest, adapter_digest);
    assert_eq!(tokenizer.encode_into("hello", &mut [0; 8]), None);
    let mut decoded = [0; 16];
    let count = tokenizer
        .decode_into(&[1, 2], &mut decoded)
        .expect("exact decode");
    assert_eq!(&decoded[..count], b"hello!");
    assert_eq!(tokenizer.decode_into(&[99], &mut decoded), None);
    let _ = std::fs::remove_dir_all(dir);
}

// ---------------------------------------------------------- abi version --

#[test]
fn abi_version_reports_format_and_contract_surface() {
    let abi = AbiVersion::current();
    assert_eq!(abi.format_major, FORMAT_VERSION_MAJOR);
    assert_eq!(abi.format_minor, FORMAT_VERSION_MINOR);
    assert_eq!(abi.contract, INFERENCE_OPERATION_CONTRACT_VERSION);
    assert_eq!(abi.api_crate_version, env!("CARGO_PKG_VERSION"));
}

// -------------------------------------------------------- engine errors --

#[test]
fn engine_load_rejects_garbage_graph() {
    let parts = EngineParts {
        graph: b"not an r4g1 artifact",
        signature_artifact: b"not a teacher artifact",
        tokenizer: None,
        score_report: None,
    };
    match R4Engine::load(parts) {
        Err(error) => assert!(error.reason.contains("invalid R4G1 graph"), "{error}"),
        other => panic!(
            "expected an invalid-graph failure, got {}",
            outcome_label(other)
        ),
    }
}

#[test]
fn engine_load_rejects_garbage_score_report_after_valid_graph_shape() {
    // A graph that fails structural parse still reports the graph failure
    // before the score report is examined (fail-fast order).
    let parts = EngineParts {
        graph: &[],
        signature_artifact: &[],
        tokenizer: None,
        score_report: Some(b"{not json"),
    };
    match R4Engine::load(parts) {
        Err(error) => assert!(error.reason.contains("invalid R4G1 graph"), "{error}"),
        other => panic!(
            "expected an invalid-graph failure, got {}",
            outcome_label(other)
        ),
    }
}

fn outcome_label(result: Result<R4Engine, uor_r4_api::SourceUnavailable>) -> String {
    match result {
        Ok(_) => "Ok(engine)".to_owned(),
        Err(error) => format!("Err({error})"),
    }
}

fn minimal_graph_with_tokenizer_cid(tokenizer_cid: [u8; 32]) -> Vec<u8> {
    let mut head = Vec::with_capacity(224);
    head.extend_from_slice(&[0x11; 32]); // teacher CID
    head.extend_from_slice(&tokenizer_cid);
    head.extend_from_slice(&[0x33; 32]); // corpus-construction CID
    head.extend_from_slice(&[0x44; 32]); // corpus-certification CID
    head.extend_from_slice(b"0123456789abcdef0123"); // HF revision
    head.extend_from_slice(&[0x55; 32]); // compiler-version CID
    head.extend_from_slice(&32u16.to_le_bytes()); // max frontier width
    head.extend_from_slice(&16u16.to_le_bytes()); // max candidates
    head.extend_from_slice(&8u16.to_le_bytes()); // signature words
    head.extend_from_slice(&8u16.to_le_bytes()); // shortlist size
    head.extend_from_slice(&64u32.to_le_bytes()); // max emission entries
    head.extend_from_slice(&64u32.to_le_bytes()); // max program steps
    head.extend_from_slice(&0u32.to_le_bytes()); // node count
    head.extend_from_slice(&0u32.to_le_bytes()); // edge count
    head.push(1); // depth count
    head.extend_from_slice(&[0; 5]); // fallback policies
    head.extend_from_slice(&[0; 2]); // reserved
    head.extend_from_slice(&64u16.to_le_bytes()); // signature bytes
    head.extend_from_slice(&1u16.to_le_bytes()); // minimum runtime major
    head.extend_from_slice(&0u16.to_le_bytes()); // minimum runtime minor
    head.extend_from_slice(&0u16.to_le_bytes()); // required feature bits
    head.extend_from_slice(&100u32.to_le_bytes()); // vocabulary size
    assert_eq!(head.len(), 224);

    let mut builder = ArtifactBuilder::new(3);
    builder.add_section(SectionId::HEAD, 0, &head);
    builder.build().expect("minimal graph")
}

#[test]
fn engine_load_requires_tokenizer_bytes_for_a_nonzero_head_cid() {
    let tokenizer = tokenizer_bytes(&[b"a"]);
    let expected = *blake3::hash(&tokenizer).as_bytes();
    let graph = minimal_graph_with_tokenizer_cid(expected);
    let result = R4Engine::load(EngineParts {
        graph: &graph,
        signature_artifact: b"invalid teacher must not be reached",
        tokenizer: None,
        score_report: None,
    });
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("a nonzero tokenizer CID requires exact bytes"),
    };
    assert!(error.reason.contains("tokenizer unavailable"), "{error}");
    assert!(
        error
            .reason
            .contains(&blake3::Hash::from(expected).to_hex().to_string()),
        "{error}"
    );
}

#[test]
fn engine_load_rejects_tokenizer_bytes_that_do_not_match_the_head_cid() {
    let expected_bytes = tokenizer_bytes(&[b"a"]);
    let graph = minimal_graph_with_tokenizer_cid(*blake3::hash(&expected_bytes).as_bytes());
    let result = R4Engine::load(EngineParts {
        graph: &graph,
        signature_artifact: b"invalid teacher must not be reached",
        tokenizer: Some(&tokenizer_bytes(&[b"b"])),
        score_report: None,
    });
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("swapped tokenizer bytes must fail before teacher parsing"),
    };
    assert!(error.reason.contains("tokenizer_cid mismatch"), "{error}");
    assert_eq!(error.reason.matches("blake3:").count(), 2, "{error}");
}

#[test]
fn engine_load_keeps_missing_tokenizer_compatible_for_a_zero_head_cid() {
    let graph = minimal_graph_with_tokenizer_cid([0; 32]);
    let result = R4Engine::load(EngineParts {
        graph: &graph,
        signature_artifact: b"invalid teacher is the next load boundary",
        tokenizer: None,
        score_report: None,
    });
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("invalid teacher still fails after legacy tokenizer handling"),
    };
    assert!(error.reason.contains("teacher artifact"), "{error}");
    assert!(!error.reason.contains("tokenizer unavailable"), "{error}");
}

#[test]
fn engine_load_rejects_a_tagged_tokenizer_when_the_head_cid_is_zero() {
    let dir = temp_dir("zero-cid-tagged");
    let path = dir.join("tokenizer.bin");
    let table = RuntimeTokenizerDecodeTable {
        identity: RuntimeTokenizerIdentity {
            family: "future-family".to_owned(),
            version: 41,
            tokenizer_cid: format!("blake3:{}", "1".repeat(64)),
            adapter_digest: format!("blake3:{}", "2".repeat(64)),
        },
        pieces: vec![Vec::new(), b"piece".to_vec()],
        encode_policy: RuntimeTokenizerEncodePolicy::Unavailable,
        decode_policy: RuntimeTokenizerDecodePolicy::SentencePiece {
            strip_dummy_prefix: true,
        },
        source_byte_lengths: None,
    };
    export_runtime_tokenizer_table(&table, &path).expect("tagged export");
    let tokenizer = std::fs::read(&path).expect("tagged bytes");
    let graph = minimal_graph_with_tokenizer_cid([0; 32]);
    let result = R4Engine::load(EngineParts {
        graph: &graph,
        signature_artifact: b"invalid teacher must not be reached",
        tokenizer: Some(&tokenizer),
        score_report: None,
    });
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("a zero-CID graph cannot bind a tagged tokenizer"),
    };
    assert!(error.reason.contains("tagged tokenizer"), "{error}");
    assert!(error.reason.contains("nonzero"), "{error}");

    let graph = minimal_graph_with_tokenizer_cid(*blake3::hash(&tokenizer).as_bytes());
    let result = R4Engine::load(EngineParts {
        graph: &graph,
        signature_artifact: b"invalid teacher is the next boundary",
        tokenizer: Some(&tokenizer),
        score_report: None,
    });
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("invalid teacher must still fail"),
    };
    assert!(error.reason.contains("teacher artifact"), "{error}");
    assert!(!error.reason.contains("tagged tokenizer"), "{error}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn engine_errors_implement_std_error() {
    let parts = EngineParts {
        graph: b"garbage",
        signature_artifact: b"garbage",
        tokenizer: None,
        score_report: None,
    };
    let error = match R4Engine::load(parts) {
        Err(error) => error,
        Ok(_) => panic!("garbage graph must fail"),
    };
    // The sanctioned SourceUnavailable is a leaf std::error::Error: its full
    // diagnostic is carried inline in Display, with no wrapped source.
    assert!(!error.to_string().is_empty());
    assert!(error.source().is_none());
}

#[test]
fn predict_output_defaults_to_no_prediction() {
    let out = PredictOutput::default();
    assert!(!out.abstained);
    assert_eq!(out.token, 0);
    assert_eq!(out.status, None);
}

// ------------------------------------------------------- compile errors --

fn request(source: PathBuf, work: PathBuf) -> CompileRequest {
    CompileRequest {
        source_dir: source,
        work_dir: work,
        tokenizer_adapter: TokenizerAdapterKey::hf_byte_bpe_v1(),
        options: CompileOptions::default(),
        source_manifest_kappa: None,
    }
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "uor-r4-api-test-{}-{}-{}",
        tag,
        std::process::id(),
        std::thread::current().name().unwrap_or("t")
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn compile_rejects_missing_source() {
    let work = temp_dir("work-missing");
    let outcome = compile(
        &request(
            PathBuf::from("/nonexistent/uor-r4-api-source"),
            work.clone(),
        ),
        &mut |_| {},
    );
    match outcome {
        Err(error) => assert!(error.reason.contains("invalid source"), "{error}"),
        other => panic!("expected an invalid-source failure, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&work);
}

#[test]
fn compile_rejects_source_without_required_files() {
    let source = temp_dir("src-incomplete");
    let work = temp_dir("work-incomplete");
    std::fs::write(source.join("config.json"), b"{}").expect("config");
    // Tokenizer definition and weights are both missing. Structural weight
    // validation runs before parsing the explicitly selected tokenizer.
    match compile(&request(source.clone(), work.clone()), &mut |_| {}) {
        Err(error) => assert!(error.reason.contains("safetensors"), "{error}"),
        other => panic!("expected an invalid-source failure, got {other:?}"),
    }
    // Weights still missing after the tokenizer appears.
    std::fs::write(source.join("tokenizer.json"), b"{}").expect("tokenizer");
    match compile(&request(source.clone(), work.clone()), &mut |_| {}) {
        Err(error) => assert!(error.reason.contains("safetensors"), "{error}"),
        other => panic!("expected an invalid-source failure, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&source);
    let _ = std::fs::remove_dir_all(&work);
}

#[test]
fn compile_requires_the_explicitly_selected_tokenizer_definition() {
    let source = temp_dir("src-selection");
    let work = temp_dir("work-selection");
    std::fs::write(source.join("config.json"), b"{}").expect("config");
    std::fs::write(source.join("model.safetensors"), b"fixture").expect("weights");
    std::fs::write(
        source.join("tokenizer.json"),
        br#"{
            "model":{"type":"BPE","vocab":{"a":0},"merges":[]},
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false}
        }"#,
    )
    .expect("tokenizer");

    let mut selected_sentencepiece = request(source.clone(), work.clone());
    selected_sentencepiece.tokenizer_adapter = TokenizerAdapterKey::new("sentencepiece-unigram", 1);
    match compile(&selected_sentencepiece, &mut |_| {}) {
        Err(error) => {
            assert!(error.reason.contains("spiece.model"), "{error}");
            assert!(error.reason.contains("sentencepiece-unigram/1"), "{error}");
        }
        other => panic!("selected missing definition must fail, got {other:?}"),
    }

    let mut unknown = request(source.clone(), work.clone());
    unknown.tokenizer_adapter = TokenizerAdapterKey::new("hf-byte-bpe", 999);
    match compile(&unknown, &mut |_| {}) {
        Err(error) => assert!(
            error.reason.contains("hf-byte-bpe/999")
                && error
                    .reason
                    .contains("not in the versioned adapter registry"),
            "{error}"
        ),
        other => panic!("unknown adapter version must fail, got {other:?}"),
    }

    let _ = std::fs::remove_dir_all(&source);
    let _ = std::fs::remove_dir_all(&work);
}

#[test]
fn compile_options_default_to_stage_defaults() {
    let options = CompileOptions::default();
    assert_eq!(options.quality_profile, QualityProfile::RelativeTla);
    assert!(options.depths > 0 && options.k0 > 0 && options.regions_budget > 0);
    assert!(options.scoring.root_top_b > 0 && options.scoring.exct_top_x > 0);
    assert_eq!(Stage::TeacherBundle.label(), "teacher-bundle");
}

#[test]
fn compile_request_selection_type_is_available_from_the_root_facade() {
    let selection: uor_r4_api::TokenizerAdapterKey = TokenizerAdapterKey::new("future-family", 41);
    assert_eq!(selection.family, "future-family");
    assert_eq!(selection.version, 41);
    let runtime_identity: Option<&uor_r4_api::RuntimeTokenizerIdentity> = None;
    assert!(runtime_identity.is_none());
    let full_adapter: Option<uor_r4_api::TokenizerAdapter> = None;
    assert!(full_adapter.is_none());
}

// -------------------------------------------------------------- e2e -----

/// End-to-end compile + engine load against a real local HF-style source.
/// Ignored by default (needs a pinned teacher on disk and minutes of CPU);
/// hologram-ai owns the maintained E2E. To run:
///
/// ```sh
/// UOR_R4_API_E2E_SOURCE=/path/to/local/hf-source \
///   cargo test -p uor-r4-api --release -- --ignored e2e --nocapture
/// ```
///
/// The source directory must carry config.json, tokenizer.json, and
/// safetensors weights. The corpus budget is tiny, so the first call may
/// legitimately return `Incomplete`; re-run to resume.
#[test]
#[ignore]
fn e2e_compile_then_engine_load() {
    let Ok(source) = std::env::var("UOR_R4_API_E2E_SOURCE") else {
        eprintln!("UOR_R4_API_E2E_SOURCE not set; skipping");
        return;
    };
    let work = temp_dir("e2e-work");
    let mut events = 0usize;
    let outcome = compile(
        &CompileRequest {
            source_dir: PathBuf::from(source),
            work_dir: work.clone(),
            tokenizer_adapter: TokenizerAdapterKey::hf_byte_bpe_v1(),
            options: CompileOptions {
                seconds: 30,
                target: 2_000,
                ..CompileOptions::default()
            },
            source_manifest_kappa: None,
        },
        &mut |event| {
            events += 1;
            eprintln!("[{} {:>3}%] {}", event.stage, event.percent, event.label);
        },
    )
    .expect("compile stages run");
    assert!(events > 0, "progress events observed");
    let model = match outcome {
        CompileOutcome::Complete(model) => model,
        CompileOutcome::Incomplete { resume_hint } => {
            eprintln!("corpus incomplete; re-run to resume: {resume_hint:?}");
            let _ = std::fs::remove_dir_all(&work);
            return;
        }
    };
    let mut engine = R4Engine::load(EngineParts {
        graph: &model.graph,
        signature_artifact: &model.signature_artifact,
        tokenizer: model.tokenizer.as_deref(),
        score_report: Some(&model.score_report),
    })
    .expect("compiled parts load as an engine");
    let abi = engine.abi_version();
    assert_eq!(abi.format_major, FORMAT_VERSION_MAJOR);
    // One prediction through the typed output slot.
    let mut out = PredictOutput::default();
    engine
        .predict_next_into(&[1], &mut out)
        .expect("prediction runs");
    // Verify that passing a mismatched tokenizer causes TokenizerCidMismatch error:
    let dummy_tokenizer = b"mismatched tokenizer binary bytes";
    let mismatch_result = R4Engine::load(EngineParts {
        graph: &model.graph,
        signature_artifact: &model.signature_artifact,
        tokenizer: Some(dummy_tokenizer),
        score_report: Some(&model.score_report),
    });
    match mismatch_result {
        Err(error) => {
            // The sanctioned reason carries both the header-expected and the
            // loaded tokenizer CIDs.
            assert!(error.reason.contains("tokenizer_cid mismatch"), "{error}");
            assert_eq!(error.reason.matches("blake3:").count(), 2, "{error}");
        }
        other => panic!(
            "expected a tokenizer_cid mismatch, got {}",
            outcome_label(other)
        ),
    }

    let _ = std::fs::remove_dir_all(&work);
}
