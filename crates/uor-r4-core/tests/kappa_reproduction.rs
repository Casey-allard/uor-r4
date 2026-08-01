//! κ-reproduction acceptance test for the transformerless → R4 integration
//! migration: the ported pipeline must reproduce every artifact κ of the
//! pre-migration baseline (tests/fixtures/baseline_kappa.json), bit
//! identically, on this platform. This is the migration proof (PROOF.md P3):
//! the port is behaviorally identical iff all pins match.
//!
//! Ignored by default: it needs the source checkpoint (60 MB, see
//! `transformerless setup`) and a release build for sane runtime. Run explicitly:
//!
//!   cargo test -p uor-r4-core --release --test kappa_reproduction -- --ignored
//!
//! Checkpoint path override: TLESS_CHECKPOINT=/path/to/model.bin

use uor_r4_core::transformerless::compiler;
use uor_r4_model_source::{LlamaOracle, TeacherOracle};

fn kappa_of(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn strings(v: &serde_json::Value, key: &str) -> Vec<String> {
    v[key]
        .as_array()
        .unwrap_or_else(|| panic!("baseline key {key} is not an array"))
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

#[test]
#[ignore]
fn kappa_reproduction() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let ckpt =
        std::env::var("TLESS_CHECKPOINT").unwrap_or_else(|_| "/tmp/ref/out/model.bin".to_string());
    if std::fs::metadata(&ckpt).is_err() {
        eprintln!("skipping: source checkpoint not found at {ckpt} (see `transformerless setup`)");
        return;
    }
    let baseline: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{dir}/tests/fixtures/baseline_kappa.json")).unwrap(),
    )
    .unwrap();
    let corpus = compiler::load_corpus_from(
        &format!("{dir}/tests/fixtures/c_meta.bin"),
        &format!("{dir}/tests/fixtures/c_recs.bin"),
    )
    .expect("corpus fixtures load");
    let oracle = LlamaOracle::load(&ckpt);
    let art = compiler::compile(&oracle, &corpus);

    // Source pin.
    assert_eq!(
        oracle.kappa(),
        baseline["source"]["kappa"].as_str().unwrap(),
        "source κ"
    );

    // Token side — derives only from the embedding table; platform-independent.
    assert_eq!(
        art.token_stage_kappas,
        strings(&baseline, "token_codebook_stages"),
        "token codebook stage κs"
    );
    let books: Vec<String> = art
        .stage_books
        .iter()
        .map(|b| kappa_of(&b.iter().map(|&x| x as u8).collect::<Vec<u8>>()))
        .collect();
    assert_eq!(books, strings(&baseline, "stage_books"), "stage book κs");
    assert_eq!(
        kappa_of(&art.token_codes),
        baseline["token_codes"].as_str().unwrap(),
        "token codes κ"
    );
    let shifts: Vec<u64> = art.stage_shifts.iter().map(|&s| s as u64).collect();
    let want_shifts: Vec<u64> = baseline["stage_shifts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_u64().unwrap())
        .collect();
    assert_eq!(shifts, want_shifts, "decode shifts");

    // Bundle-derived — platform-sensitive section (macOS pins).
    let bd = &baseline["bundle_derived_macos"];
    let thr: Vec<u8> = art
        .thresholds
        .iter()
        .flat_map(|t| t.to_le_bytes())
        .collect();
    assert_eq!(
        kappa_of(&thr),
        bd["threshold_vector"].as_str().unwrap(),
        "threshold vector κ"
    );
    let ctx: Vec<String> = art
        .ctx_cb
        .iter()
        .map(|cb| compiler::kappa_of_f32s(cb))
        .collect();
    assert_eq!(
        ctx,
        strings(bd, "context_codebook_stages"),
        "context codebook κs"
    );
    let sigs: Vec<String> = art.class_sigs.iter().map(|s| kappa_of(s)).collect();
    assert_eq!(sigs, strings(bd, "class_signatures"), "class signature κs");

    // Container (TLA5) — byte length and κ.
    let container = compiler::artifact_bytes(&art);
    assert_eq!(
        container.len() as u64,
        bd["container"]["bytes"].as_u64().unwrap(),
        "container byte length"
    );
    assert_eq!(
        kappa_of(&container),
        bd["container"]["kappa"].as_str().unwrap(),
        "container κ"
    );
}

/// Re-pinning helper: compiles against the same fixtures and prints a complete
/// baseline JSON (asserted fields) on stdout. Use when the compiler is
/// intentionally redesigned and the pins must move:
///
///   cargo test -p uor-r4-core --release --test kappa_reproduction -- \
///     --ignored --nocapture dump_baseline_kappa > /tmp/new_baseline.json
///
/// then review the diff against tests/fixtures/baseline_kappa.json before
/// adopting (a maintainer decision, never automatic).
#[test]
#[ignore]
fn dump_baseline_kappa() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let ckpt =
        std::env::var("TLESS_CHECKPOINT").unwrap_or_else(|_| "/tmp/ref/out/model.bin".to_string());
    if std::fs::metadata(&ckpt).is_err() {
        eprintln!("skipping: source checkpoint not found at {ckpt}");
        return;
    }
    let corpus = compiler::load_corpus_from(
        &format!("{dir}/tests/fixtures/c_meta.bin"),
        &format!("{dir}/tests/fixtures/c_recs.bin"),
    )
    .expect("corpus fixtures load");
    let oracle = LlamaOracle::load(&ckpt);
    let art = compiler::compile(&oracle, &corpus);

    let books: Vec<String> = art
        .stage_books
        .iter()
        .map(|b| kappa_of(&b.iter().map(|&x| x as u8).collect::<Vec<u8>>()))
        .collect();
    let thr: Vec<u8> = art
        .thresholds
        .iter()
        .flat_map(|t| t.to_le_bytes())
        .collect();
    let ctx: Vec<String> = art
        .ctx_cb
        .iter()
        .map(|cb| compiler::kappa_of_f32s(cb))
        .collect();
    let sigs: Vec<String> = art.class_sigs.iter().map(|s| kappa_of(s)).collect();
    let container = compiler::artifact_bytes(&art);

    let out = serde_json::json!({
        "source": { "kappa": oracle.kappa(), "bytes": oracle.source_bytes() },
        "token_codebook_stages": art.token_stage_kappas,
        "stage_books": books,
        "token_codes": kappa_of(&art.token_codes),
        "stage_shifts": art.stage_shifts.iter().map(|&s| s as u64).collect::<Vec<_>>(),
        "bundle_derived_macos": {
            "threshold_vector": kappa_of(&thr),
            "context_codebook_stages": ctx,
            "class_signatures": sigs,
            "container": { "bytes": container.len() as u64, "kappa": kappa_of(&container) },
        },
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());

    // With TLESS_REPIN_WRITE=1, also regenerate the fixture container that
    // window_paths::container_roundtrip_byte_identical pins against the
    // baseline κ. Only run alongside an intentional re-pin.
    if std::env::var("TLESS_REPIN_WRITE").as_deref() == Ok("1") {
        let fixture = format!("{dir}/tests/fixtures/tless_artifacts.bin");
        std::fs::write(&fixture, &container).expect("write fixture container");
        eprintln!("wrote {} bytes to {fixture}", container.len());
    }
}

/// #318 Phase B witness on a REAL compile: the TLA7 residual-wired
/// assignment must agree across every form of the path — kernel
/// (`Runtime::assign`), the plain bundle entry points (`code_plain`,
/// `assign_for_bundle`, membership primary), and the allocation-free
/// serving form (`assign_code_for_bundle`) — over hundreds of corpus
/// positions, on both the in-memory compile and the serialized → parsed
/// TLA7 container. Regression test for the kernel-residual vs
/// plain-non-residual routing divergence the first Phase B quality run
/// hit (certify.rs "code kernel/plain divergence"). Same harness and
/// skip convention as the κ-reproduction test above.
#[test]
#[ignore]
fn tla7_resid_kernel_plain_witness() {
    use uor_r4_core::transformerless::runtime;

    let dir = env!("CARGO_MANIFEST_DIR");
    let ckpt =
        std::env::var("TLESS_CHECKPOINT").unwrap_or_else(|_| "/tmp/ref/out/model.bin".to_string());
    if std::fs::metadata(&ckpt).is_err() {
        eprintln!("skipping: source checkpoint not found at {ckpt} (see `transformerless setup`)");
        return;
    }
    let corpus = compiler::load_corpus_from(
        &format!("{dir}/tests/fixtures/c_meta.bin"),
        &format!("{dir}/tests/fixtures/c_recs.bin"),
    )
    .expect("corpus fixtures load");
    let oracle = LlamaOracle::load(&ckpt);
    let art = compiler::compile(&oracle, &corpus);
    assert!(
        !art.resid_cb.is_empty(),
        "fresh compile carries the TLA7 residual sections"
    );
    let parsed = compiler::parse_artifacts(&compiler::artifact_bytes(&art))
        .expect("TLA7 container round-trips");
    assert_eq!(parsed.resid_cb, art.resid_cb);
    assert_eq!(parsed.resid_scale_shifts, art.resid_scale_shifts);
    assert_eq!(parsed.norm_fold_const, art.norm_fold_const);

    let rot = compiler::derive_rotations();
    for (label, a) in [("in-memory", &art), ("parsed-tla7", &parsed)] {
        let mut rt = runtime::Runtime::new(a);
        let sample_n = 512usize;
        let stride = corpus.n / sample_n;
        for s in 0..sample_n {
            let i = s * stride;
            let bk = runtime::bundle_kernel(&mut rt.kernel, a, &rot, &corpus, i);
            let bp = runtime::bundle_plain(a, &rot, &corpus, i);
            assert_eq!(bk, bp, "[{label}] bundle kernel/plain divergence at {i}");
            let ck = rt.assign(&corpus, i);
            let cp = runtime::code_plain(a, &rot, &corpus, i);
            assert_eq!(ck, cp, "[{label}] code kernel/plain divergence at {i}");
            assert_eq!(
                runtime::assign_for_bundle(a, &bp),
                cp,
                "[{label}] assign_for_bundle divergence at {i}"
            );
            assert_eq!(
                runtime::assign_code_for_bundle(a, &bp),
                cp,
                "[{label}] serving-form divergence at {i}"
            );
            let (primary, _) = runtime::assign_memberships_for_bundle(a, &bp);
            assert_eq!(
                primary, cp,
                "[{label}] membership primary divergence at {i}"
            );
        }
        println!(
            "[{label}] TLA7 residual witness: {sample_n}/{sample_n} positions, all forms agree"
        );
    }
}
