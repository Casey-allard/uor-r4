//! **CM-EXT — external real-model validation & verification.**
//!
//! Pins published GGUF / ONNX models by download URL, file SHA-256, and
//! the κ-label `uor_addr::{gguf,onnx}::address` mints for them. For each
//! pinned model the test:
//!
//!   1. ensures the model is cached locally, downloading it on first run
//!      and **verifying the bytes against the pinned file SHA-256** (so a
//!      corrupted / substituted download is rejected — the pin *is* the
//!      content-address of the input file);
//!   2. asserts the Rust `address()` κ-label equals the pinned label;
//!   3. asserts the **independent Python reference encoder**
//!      (`tools/canonical-{gguf,onnx}.py`) produces the same label
//!      (cross-implementation attestation of the canonical form);
//!   4. asserts the ψ-pipeline **carrier (canonical skeleton) stays
//!      bounded** — for the large GGUF the skeleton is a few tens of KB
//!      against a ~531 MB model, proving the streaming property: every
//!      weight byte binds (via streamed per-tensor digests) while the
//!      carrier size depends only on the tensor / KV counts, not on model
//!      size; and
//!   5. round-trips the owned TC-05 witness (`verify()`).
//!
//! Gated behind `UOR_ADDR_LIVE=1` (run via `just cn`): the downloads need
//! network access and total ~635 MB. The models are cached under
//! `tests/fixtures/models/` (git-ignored) and reused across runs.
//!
//! The 531 MB Qwen2-0.5B GGUF is the **large-model streaming** vector the
//! V&V plan requires; the two ONNX models (IR v3 and IR v6) exercise the
//! IR-version range the realization now admits (`1..=ONNX_IR_VERSION_MAX`).

#![cfg(all(feature = "gguf", feature = "onnx"))]

use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Copy)]
enum Format {
    Gguf,
    Onnx,
}

struct Pinned {
    /// Cache file name (under `tests/fixtures/models/`).
    name: &'static str,
    /// Stable download URL.
    url: &'static str,
    /// Lowercase-hex SHA-256 of the file bytes (the input's own content
    /// address — pins the exact model revision).
    file_sha256: &'static str,
    /// The κ-label `address()` mints for this model.
    kappa: &'static str,
    format: Format,
}

/// The pinned external-model corpus. κ-labels + SHA-256 are recorded from
/// the live download; re-run `just cn` after bumping a pin.
const MODELS: &[Pinned] = &[
    // Large LLM weights in GGUF v3 (Q8_0) — the streaming vector
    // (531 MB model → ~28 KB canonical skeleton carrier).
    Pinned {
        name: "qwen2-0_5b-instruct-q8_0.gguf",
        url: "https://huggingface.co/Qwen/Qwen2-0.5B-Instruct-GGUF/resolve/main/qwen2-0_5b-instruct-q8_0.gguf",
        file_sha256: "834f4115ad5a836c9f17716b1577290fda96de3deb881ba45a4d5476fd202e96",
        kappa: "sha256:66c2ea8fa51317c6da91d10f131a5de64d45cb859edaf7a4f8d2557277f45b2d",
        format: Format::Gguf,
    },
    // ONNX IR v3 (opset 7), inline `raw_data` weights.
    Pinned {
        name: "mobilenetv2-7.onnx",
        url: "https://media.githubusercontent.com/media/onnx/models/main/validated/vision/classification/mobilenet/model/mobilenetv2-7.onnx",
        file_sha256: "c1c513582d56afceff8516c73804e484c81c6a830712ab6d682253f4a3cd042f",
        kappa: "sha256:f71c815228869e2c56ad00fcf4691ffbad45ecb72a503eef35cbaabe40287378",
        format: Format::Onnx,
    },
    // ONNX IR v6, a transformer encoder (different structure + IR than
    // the mobilenet vector).
    Pinned {
        name: "all-MiniLM-L6-v2.onnx",
        url: "https://huggingface.co/Xenova/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx",
        file_sha256: "759c3cd2b7fe7e93933ad23c4c9181b7396442a2ed746ec7c1d46192c469c46e",
        kappa: "sha256:a036c7fec3409bb71116dcf79a37ce368166898b8621a75bc19299340e127422",
        format: Format::Onnx,
    },
];

fn live() -> bool {
    std::env::var("UOR_ADDR_LIVE").as_deref() == Ok("1")
}

fn cache_dir() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/models"
    ))
}

fn file_sha256_hex(path: &PathBuf) -> String {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .expect("run sha256sum");
    assert!(out.status.success(), "sha256sum failed");
    String::from_utf8(out.stdout).unwrap()[..64].to_string()
}

/// Download the model on first run; verify the cached bytes against the
/// pinned file SHA-256 on every run.
fn ensure_cached(m: &Pinned) -> PathBuf {
    let path = cache_dir().join(m.name);
    if !path.exists() {
        std::fs::create_dir_all(cache_dir()).unwrap();
        let status = Command::new("curl")
            .args(["-sL", "--fail", "-o"])
            .arg(&path)
            .arg(m.url)
            .status()
            .expect("run curl");
        assert!(status.success(), "download failed for {}", m.name);
    }
    let got = file_sha256_hex(&path);
    assert_eq!(
        got, m.file_sha256,
        "{}: cached file SHA-256 does not match the pin — corrupted or \
         substituted model (delete the cache file to re-download)",
        m.name
    );
    path
}

fn python_label(m: &Pinned, path: &PathBuf) -> String {
    let tool = match m.format {
        Format::Gguf => concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/canonical-gguf.py"),
        Format::Onnx => concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/canonical-onnx.py"),
    };
    let out = Command::new("python3")
        .arg(tool)
        .arg(path)
        .output()
        .expect("run python canonical tool");
    assert!(
        out.status.success(),
        "python tool failed for {}: {}",
        m.name,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

#[test]
#[ignore = "CM-EXT: requires UOR_ADDR_LIVE=1 + network (~635 MB) + python3"]
fn pinned_external_models_validate_and_verify() {
    if !live() {
        return;
    }
    for m in MODELS {
        let path = ensure_cached(m);
        let bytes = std::fs::read(&path).unwrap();

        let (outcome, skeleton) = match m.format {
            Format::Gguf => (
                uor_addr::gguf::address(&bytes).expect("gguf address"),
                uor_addr::gguf::canonicalize(&bytes).expect("gguf canonicalize"),
            ),
            Format::Onnx => (
                uor_addr::onnx::address(&bytes).expect("onnx address"),
                uor_addr::onnx::canonicalize(&bytes).expect("onnx canonicalize"),
            ),
        };

        // (2) Rust κ-label == pin.
        assert_eq!(
            outcome.address.as_str(),
            m.kappa,
            "{}: Rust κ-label",
            m.name
        );

        // (3) Independent Python reference == pin.
        assert_eq!(
            python_label(m, &path),
            m.kappa,
            "{}: Python κ-label",
            m.name
        );

        // (4) Bounded carrier: the canonical skeleton is the ψ-pipeline
        // carrier; for large models it is orders of magnitude smaller than
        // the model (every weight byte still binds via streamed digests).
        // A ≥1 MiB model must skeletonize below 5% of its size.
        if bytes.len() >= (1 << 20) {
            assert!(
                skeleton.len() * 20 < bytes.len(),
                "{}: skeleton {} not << model {} (streaming/bounded-carrier property)",
                m.name,
                skeleton.len(),
                bytes.len()
            );
        }

        // (5) Owned TC-05 witness round-trips.
        assert_eq!(
            outcome.witness.verify().expect("replay verify"),
            outcome.address,
            "{}: witness verify",
            m.name
        );
    }
}
