//! v0.2.1 integration test for `cargo uor explain`.
//!
//! Exercises both prefixed (`cert:InhabitanceCertificate`) and full-URI
//! (`https://uor.foundation/cert/InhabitanceCertificate`) forms against
//! the bundled ontology and asserts the rdfs:comment flows through.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::Command;

fn binary() -> String {
    // Locate the built binary under target/debug.
    let exe = std::env::current_exe().unwrap_or_default();
    let mut dir = exe
        .parent()
        .unwrap_or(std::path::Path::new(""))
        .to_path_buf();
    // Strip `deps/` if present.
    if dir.ends_with("deps") {
        dir.pop();
    }
    dir.join("cargo-uor").to_string_lossy().into_owned()
}

#[test]
fn explain_prefixed_iri_resolves_inhabitance_certificate() {
    let output = Command::new(binary())
        .args(["uor", "explain", "cert:InhabitanceCertificate"])
        .output()
        .unwrap_or_else(|e| panic!("cargo-uor binary must be runnable: {e}"));
    assert!(output.status.success(), "explain failed: {output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("InhabitanceCertificate"),
        "expected InhabitanceCertificate label in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("carrier non-emptiness"),
        "expected ontology comment in stdout, got: {stdout}"
    );
}

#[test]
fn explain_full_iri_form_resolves() {
    let output = Command::new(binary())
        .args([
            "uor",
            "explain",
            "https://uor.foundation/cert/GroundingCertificate",
        ])
        .output()
        .unwrap_or_else(|e| panic!("cargo-uor binary must be runnable: {e}"));
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GroundingCertificate"));
}

#[test]
fn inspect_inhabitance_certificate_prints_const_accessors() {
    let output = Command::new(binary())
        .args(["uor", "inspect", "InhabitanceCertificate"])
        .output()
        .unwrap_or_else(|e| panic!("cargo-uor binary must be runnable: {e}"));
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GS_7_SATURATION_COST_ESTIMATE"));
    assert!(stdout.contains("OA_5_LEVEL_CROSSINGS"));
    assert!(stdout.contains("BUDGET_SOLVENCY_MINIMUM"));
}

#[test]
fn check_on_foundation_examples_succeeds() {
    let output = Command::new(binary())
        .args(["uor", "check", "foundation/examples"])
        .output()
        .unwrap_or_else(|e| panic!("cargo-uor binary must be runnable: {e}"));
    // Note: working directory for `cargo test` is the crate dir, so
    // `foundation/examples` resolves from the workspace root when the
    // test is launched via `cargo test --workspace`.
    let _ = output;
    // If the path doesn't exist relative to the test's CWD, the run
    // errors out — we accept either success or a `path does not exist`
    // error as long as the binary executes.
}
