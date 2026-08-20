//! #830 negative fixture: a missing empirical fixture serializes as
//! `UNAVAILABLE`, never `PASS`.
//!
//! The register's `build` level is harness-built (structural) status. An
//! empirical verdict is a separate axis, and its non-vacuity rule is that an
//! absent, CID-bound fixture cannot mint a `PASS` — the RF-29 hazard (a
//! teacher-parity benchmark that "vacuously passes" when its pinned fixtures are
//! absent), encoded as `repo_model::EmpiricalStatus`. This planted negative
//! points the resolver at a path guaranteed not to exist and asserts the
//! serialized verdict is `UNAVAILABLE`; a positive control points it at a path
//! that does exist, so the negative is not the behaviour of an instrument that
//! can never PASS.

use std::path::PathBuf;

use repo_model::EmpiricalStatus;

/// A path under the temp dir that is guaranteed absent at resolve time.
fn absent_fixture() -> PathBuf {
    let p = std::env::temp_dir().join("uor-r4-830-absent-empirical-fixture-does-not-exist");
    // Best-effort remove in case a previous run left something behind.
    let _ = std::fs::remove_file(&p);
    let _ = std::fs::remove_dir_all(&p);
    assert!(
        !p.exists(),
        "fixture must be absent for this negative control"
    );
    p
}

/// Negative control: an absent fixture is `UNAVAILABLE` and serializes as the
/// `UNAVAILABLE` token — even when the would-be run outcome is "passing". A
/// missing fixture cannot mint an empirical certificate (#830).
#[test]
fn an_absent_empirical_fixture_serializes_unavailable_not_pass() {
    let fixture = absent_fixture();

    // Even if the run "would have passed", absence dominates.
    let status = EmpiricalStatus::for_fixture(&fixture, true);
    assert_eq!(status, EmpiricalStatus::Unavailable);
    assert_eq!(status.as_str(), "UNAVAILABLE");
    assert!(
        !status.is_pass(),
        "an absent fixture must never read as PASS"
    );

    // And with a failing would-be run, still UNAVAILABLE (nothing was measured).
    assert_eq!(
        EmpiricalStatus::for_fixture(&fixture, false).as_str(),
        "UNAVAILABLE"
    );
}

/// Positive control: a present fixture reflects the run outcome, so the negative
/// above is not vacuous. Uses a runtime-resolved repo path (never a compile-time
/// `env!` path, per the #788 cached-rlib hazard).
#[test]
fn a_present_empirical_fixture_can_pass_and_fail() {
    let present = repo_model::repo_root().join("model/ledger.toml");
    assert!(present.exists(), "positive-control fixture must exist");

    assert_eq!(
        EmpiricalStatus::for_fixture(&present, true).as_str(),
        "PASS"
    );
    assert_eq!(
        EmpiricalStatus::for_fixture(&present, false).as_str(),
        "FAIL"
    );
}
