//! Runs only when `UOR_W2_CAPTURE` (the `w2-out-b` directory) and `UOR_W2_RECORD`
//! (W2's `record.json`) are set: precedence 0 must pass, and two runs must be
//! byte-identical.

use std::path::PathBuf;

use m1_w2_code_utilization_instrument as inst;

#[test]
fn real_capture_reproduces_w2() {
    let (Ok(capture), Ok(record)) = (
        std::env::var("UOR_W2_CAPTURE"),
        std::env::var("UOR_W2_RECORD"),
    ) else {
        eprintln!("skipped: set UOR_W2_CAPTURE and UOR_W2_RECORD to run");
        return;
    };
    let (inputs, rec) = inst::load(&PathBuf::from(capture), &PathBuf::from(record)).unwrap();
    let commit = "0000000000000000000000000000000000000000";
    let first = inst::run(&inputs, &rec, commit).unwrap();
    let second = inst::run(&inputs, &rec, commit).unwrap();
    let a = serde_json::to_string(&first).unwrap();
    let b = serde_json::to_string(&second).unwrap();
    assert_eq!(a, b, "two runs differ");
    assert!(first.precedence_0.checks.iter().all(|c| c.ok));
    assert!(first.precedence_0.row_mismatches.values().all(|&n| n == 0));
    eprintln!(
        "QA {} A_W {} A_T {} A_U {} | QB {} d_T {} | QC {} J {} G {}",
        first.qa.band,
        first.qa.a_w,
        first.qa.a_t,
        first.qa.a_u,
        first.qb.result.band,
        first.qb.result.d_t,
        first.qc.band,
        first.qc.j,
        first.qc.g
    );
}
