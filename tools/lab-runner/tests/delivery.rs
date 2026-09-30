use lab_runner::delivery::*;
use lab_runner::ledger::sha256;
use serde_json::json;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
fn receipt() -> (std::path::PathBuf, DeliveryReceipt) {
    let dir = std::env::temp_dir().join(format!(
        "delivery-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("receipt.log");
    fs::write(&path, b"real local fixture output\n").unwrap();
    let file = FileEvidence {
        path,
        sha256: sha256(b"real local fixture output\n"),
    };
    let head = "a".repeat(40);
    let base = "b".repeat(40);
    let value = DeliveryReceipt {
        schema: SCHEMA.into(),
        repository: "UOR-Foundation/uor-r4".into(),
        pull_request: 42,
        task_issue: 43,
        author_session: "author".into(),
        claim_session: "author".into(),
        claim_epoch: 1,
        work_card: "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
        head_sha: head.clone(),
        base_sha: base.clone(),
        change_kind: "code".into(),
        class: "A".into(),
        checks: ["diff-check", "format", "compile", "focused-tests"]
            .iter()
            .map(|name| CheckReceipt {
                name: (*name).into(),
                head_sha: head.clone(),
                base_sha: base.clone(),
                argv: vec!["cargo".into(), "test".into()],
                exit_code: 0,
                outcome: "PASS".into(),
                log: file.clone(),
            })
            .collect(),
        reviews: vec![ReviewReceipt {
            review_id: "independent-pass-1".into(),
            reviewer_session: "reviewer".into(),
            launched_by: "another-lab".into(),
            head_sha: head,
            base_sha: base,
            decision: "APPROVE".into(),
            unresolved_required_fixes: 0,
            evidence: file,
        }],
        result_evidence: None,
        result_reader_session: None,
        council: None,
    };
    (dir, value)
}
fn live(r: &DeliveryReceipt) -> serde_json::Value {
    json!({"headRefOid":r.head_sha,"baseRefOid":r.base_sha,"baseRefName":"main","state":"OPEN","isDraft":false,"mergeStateStatus":"CLEAN","statusCheckRollup":[{"name":"compatibility acknowledgement","conclusion":"SUCCESS"}]})
}
#[test]
fn exact_receipts_and_real_queue_eligibility_are_required() {
    let (dir, r) = receipt();
    assert!(validate(&r).is_ok());
    assert!(validate_live(
        &r,
        &live(&r),
        &["compatibility acknowledgement".into()],
        true
    )
    .is_ok());
    assert!(validate_live(&r, &live(&r), &["not run".into()], true).is_err());
    assert!(validate_live(
        &r,
        &live(&r),
        &["compatibility acknowledgement".into()],
        false
    )
    .is_err());
    let mut moved = live(&r);
    moved["headRefOid"] = json!("c".repeat(40));
    assert!(validate_live(&r, &moved, &["compatibility acknowledgement".into()], true).is_err());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn stale_source_self_review_and_unresolved_fixes_fail() {
    let (dir, r) = receipt();
    let mut bad = r.clone();
    bad.reviews[0].reviewer_session = r.author_session.clone();
    assert!(validate(&bad).is_err());
    bad = r.clone();
    bad.reviews[0].unresolved_required_fixes = 1;
    assert!(validate(&bad).is_err());
    bad = r.clone();
    bad.checks[0].head_sha = "c".repeat(40);
    assert!(validate(&bad).is_err());
    bad = r.clone();
    bad.checks[0].outcome = "UNAVAILABLE".into();
    assert!(validate(&bad).is_err());
    bad = r.clone();
    bad.checks[0].exit_code = 1;
    assert!(validate(&bad).is_err());
    fs::write(&r.checks[0].log.path, b"changed").unwrap();
    assert!(validate(&r).is_err());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn result_needs_independent_artifact_reread() {
    let (dir, mut r) = receipt();
    r.class = "B".into();
    assert!(validate(&r).is_err());
    r.result_evidence = Some(r.reviews[0].evidence.clone());
    r.result_reader_session = Some("author".into());
    assert!(validate(&r).is_err());
    r.result_reader_session = Some("reviewer".into());
    assert!(validate(&r).is_ok());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn source_cannot_be_self_classified_as_docs() {
    let (dir, mut r) = receipt();
    r.change_kind = "docs".into();
    assert!(validate_scope(&r, &["tools/lab-runner/src/jobs.rs".into()]).is_err());
    assert!(validate_scope(&r, &["README.md".into()]).is_ok());
    assert!(validate_scope(&r, &["AGENTS.md".into()]).is_err());
    r.class = "C".into();
    assert!(validate_scope(&r, &["AGENTS.md".into()]).is_ok());
    r.change_kind = "code".into();
    assert!(validate_scope(&r, &["crates/uor-r4-stack/src/model.rs".into()]).is_err());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn serving_callers_and_control_plane_cannot_underclassify() {
    let (dir, mut r) = receipt();
    for path in [
        "src/native_geometric_cli.rs",
        "src/service.rs",
        "src/native_wasm.rs",
        "crates/uor-r4-api/src/serving.rs",
    ] {
        assert!(validate_scope(&r, &[path.into()]).is_err(), "{path}");
        r.change_kind = "model".into();
        assert!(validate_scope(&r, &[path.into()]).is_ok(), "{path}");
        r.change_kind = "code".into();
    }
    for path in [
        "tools/lab-runner/src/coord.rs",
        "tools/lab-runner/src/admission.rs",
        "tools/lab-runner/src/delivery.rs",
        "docs/labs/operations.md",
    ] {
        assert!(validate_scope(&r, &[path.into()]).is_err(), "{path}");
        r.class = "C".into();
        assert!(validate_scope(&r, &[path.into()]).is_ok(), "{path}");
        r.class = "A".into();
    }
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn stale_task_generation_cannot_deliver() {
    let (dir, r) = receipt();
    let mut state = json!({"schema":"uor-r4.lab-state/1","repository":r.repository,
        "labs":{"author":{"available":true,"heartbeat":100}},
        "policy_sha":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","tasks":{"43":{"policy_sha":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","session":"author","epoch":1,"work_card":r.work_card,"phase":"claimed","expires":1300}}});
    assert!(validate_claim(&r, &state, 101).is_ok());
    assert!(validate_claim(&r, &state, 1300).is_err());
    state["tasks"]["43"]["epoch"] = json!(2);
    assert!(validate_claim(&r, &state, 101).is_err());
    state["tasks"]["43"]["epoch"] = json!(1);
    state["labs"]["author"]["available"] = json!(false);
    assert!(validate_claim(&r, &state, 101).is_err());
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn council_requires_distinct_passes_and_two_nonauthors() {
    let (dir, mut r) = receipt();
    r.class = "C".into();
    assert!(validate(&r).is_err());
    r.council = Some(CouncilReceipt {
        decision_id: "policy-1".into(),
        head_sha: r.head_sha.clone(),
        base_sha: r.base_sha.clone(),
        evidence: r.reviews[0].evidence.clone(),
        votes: ["author", "reviewer", "adversary"]
            .iter()
            .map(|name| CouncilVote {
                reviewer_session: (*name).into(),
                decision: "APPROVE".into(),
                reasons: "prospective policy justified by fixture".into(),
            })
            .collect(),
    });
    assert!(validate(&r).is_ok());
    r.council.as_mut().unwrap().votes[2].reviewer_session = "reviewer".into();
    assert!(validate(&r).is_err());
    fs::remove_dir_all(dir).unwrap();
}
