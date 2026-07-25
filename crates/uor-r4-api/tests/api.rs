//! Unit tests for the typed façade: error paths, the version surface,
//! parts validation, and the bytes-based tokenizer. None of these need a
//! real teacher model; the end-to-end compile test is `#[ignore]`d (see
//! its comment) — hologram-ai owns the real E2E.

use std::error::Error;
use std::path::PathBuf;

use uor_r4_api::compile::{
    compile, CompileError, CompileOptions, CompileOutcome, CompileRequest, QualityProfile, Stage,
};
use uor_r4_api::engine::{AbiVersion, EngineParts, LoadError, PredictOutput, R4Engine};
use uor_r4_api::Tokenizer;
use uor_r4_graph_format::{
    FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR, INFERENCE_OPERATION_CONTRACT_VERSION,
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
    let error = match Tokenizer::from_bytes(&bytes) {
        Err(error) => error,
        Ok(_) => panic!("truncated token must fail"),
    };
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    // Negative length.
    let bytes = (-1i32).to_le_bytes().to_vec();
    assert!(Tokenizer::from_bytes(&bytes).is_err());

    // Trailing partial length field.
    assert!(Tokenizer::from_bytes(&[0u8, 1]).is_err());
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
        Err(LoadError::InvalidGraph(_)) => {}
        other => panic!("expected InvalidGraph, got {}", outcome_label(other)),
    }
}

#[test]
fn engine_load_rejects_garbage_score_report_after_valid_graph_shape() {
    // A graph that fails structural parse still reports InvalidGraph
    // before the score report is examined (fail-fast order).
    let parts = EngineParts {
        graph: &[],
        signature_artifact: &[],
        tokenizer: None,
        score_report: Some(b"{not json"),
    };
    match R4Engine::load(parts) {
        Err(LoadError::InvalidGraph(_)) => {}
        other => panic!("expected InvalidGraph, got {}", outcome_label(other)),
    }
}

fn outcome_label(result: Result<R4Engine, LoadError>) -> String {
    match result {
        Ok(_) => "Ok(engine)".to_owned(),
        Err(error) => format!("Err({error})"),
    }
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
    // Display is non-empty and the wrapped FormatError is chained.
    assert!(!error.to_string().is_empty());
    assert!(error.source().is_some());
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
        options: CompileOptions::default(),
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
        Err(CompileError::SourceInvalid { .. }) => {}
        other => panic!("expected SourceInvalid, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&work);
}

#[test]
fn compile_rejects_source_without_required_files() {
    let source = temp_dir("src-incomplete");
    let work = temp_dir("work-incomplete");
    std::fs::write(source.join("config.json"), b"{}").expect("config");
    // tokenizer.json and weights missing.
    match compile(&request(source.clone(), work.clone()), &mut |_| {}) {
        Err(CompileError::SourceInvalid { message }) => {
            assert!(message.contains("tokenizer.json"), "{message}");
        }
        other => panic!("expected SourceInvalid, got {other:?}"),
    }
    // Weights still missing after the tokenizer appears.
    std::fs::write(source.join("tokenizer.json"), b"{}").expect("tokenizer");
    match compile(&request(source.clone(), work.clone()), &mut |_| {}) {
        Err(CompileError::SourceInvalid { message }) => {
            assert!(message.contains("safetensors"), "{message}");
        }
        other => panic!("expected SourceInvalid, got {other:?}"),
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
            options: CompileOptions {
                seconds: 30,
                target: 2_000,
                ..CompileOptions::default()
            },
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
    assert!(out.status.is_some());
    let _ = std::fs::remove_dir_all(&work);
}
