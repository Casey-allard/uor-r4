//! Crash and resource boundaries use isolated roots only.
use lab_runner::{
    daemon::{self, DaemonConfig},
    jobs, ledger,
};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn fixture(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "uor-host-test-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}
fn spec(root: &Path, id: &str, rss: f64) -> PathBuf {
    let p = root.join("input.json");
    fs::write(&p,serde_json::to_vec(&json!({"schema":"uor-r4.lab-runner-job/1","id":id,"lab":"test","cwd":root,"argv":["/bin/sleep","20"],"threads":1,"rss_gib":rss,"wall_s":30,"kill_criterion":{"kind":"wall"}})).unwrap()).unwrap();
    p
}
fn config(root: &Path, ledger: &Path, production: bool) -> DaemonConfig {
    DaemonConfig {
        root: root.to_path_buf(),
        ledger_dir: ledger.to_path_buf(),
        poll_interval: Duration::from_millis(50),
        monitor_interval: Duration::from_millis(100),
        stop_file: Some(root.join("STOP")),
        require_host_policy: production,
    }
}
fn wait(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !path.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(30));
    }
    assert!(path.exists(), "missing {}", path.display());
}

#[test]
fn production_missing_policy_holds_without_launch() {
    let root = fixture("missing-policy");
    let ledger = root.join("ledger");
    fs::create_dir(&ledger).unwrap();
    ledger::initialize_empty(&ledger, 100_000, "test").unwrap();
    jobs::submit(&root, &spec(&root, "held", 0.25)).unwrap();
    let cfg = config(&root, &ledger, true);
    let handle = std::thread::spawn(move || daemon::run(&cfg));
    wait(&root.join("admission-blocked.json"));
    assert!(!root.join("running/held").exists());
    fs::write(root.join("STOP"), b"stop").unwrap();
    handle.join().unwrap().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn observed_rss_stops_job_and_nonzero_exit_is_not_success() {
    let root = fixture("rss");
    let ledger = root.join("ledger");
    fs::create_dir(&ledger).unwrap();
    ledger::initialize_empty(&ledger, 100_000, "test").unwrap();
    jobs::submit(&root, &spec(&root, "rss", 0.000001)).unwrap();
    let cfg = config(&root, &ledger, false);
    let handle = std::thread::spawn(move || daemon::run(&cfg));
    wait(&root.join("done/rss/exit.json"));
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("done/rss/exit.json")).unwrap()).unwrap();
    assert_eq!(receipt["outcome"], "resource_killed");
    assert_eq!(
        receipt["checkpoint_status"],
        "unavailable_no_verified_payload_protocol"
    );
    assert_eq!(receipt["checkpoint_requested"], false);
    fs::write(root.join("STOP"), b"stop").unwrap();
    handle.join().unwrap().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn background_grandchild_cannot_escape_finished_payload() {
    let root = fixture("orphan");
    let ledger = root.join("ledger");
    fs::create_dir(&ledger).unwrap();
    ledger::initialize_empty(&ledger, 100_000, "test").unwrap();
    let input = spec(&root, "orphan", 0.25);
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&input).unwrap()).unwrap();
    value["argv"] = json!(["/bin/sh", "-c", "sleep 20 &"]);
    fs::write(&input, serde_json::to_vec(&value).unwrap()).unwrap();
    jobs::submit(&root, &input).unwrap();
    let cfg = config(&root, &ledger, false);
    let handle = std::thread::spawn(move || daemon::run(&cfg));
    wait(&root.join("done/orphan/exit.json"));
    let dir = root.join("done/orphan");
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("exit.json")).unwrap()).unwrap();
    assert_eq!(receipt["outcome"], "failed");
    assert_eq!(receipt["exit_status"], 0);
    assert_eq!(receipt["process_state"], "confirmed_stopped");
    assert!(
        lab_runner::process::owned_members(&jobs::read_identity(&dir).unwrap())
            .unwrap()
            .is_empty()
    );
    fs::write(root.join("STOP"), b"stop").unwrap();
    handle.join().unwrap().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn singleton_rejects_second_daemon() {
    let root = fixture("singleton");
    let ledger = root.join("ledger");
    fs::create_dir(&ledger).unwrap();
    ledger::initialize_empty(&ledger, 100_000, "test").unwrap();
    let cfg = config(&root, &ledger, false);
    let handle = std::thread::spawn(move || daemon::run(&cfg));
    wait(&root.join("daemon.json"));
    assert!(daemon::run(&config(&root, &ledger, false)).is_err());
    fs::write(root.join("STOP"), b"stop").unwrap();
    handle.join().unwrap().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn durable_exit_replays_charge_once_after_crash() {
    let root = fixture("receipt");
    let ledger = root.join("ledger");
    fs::create_dir(&ledger).unwrap();
    ledger::initialize_empty(&ledger, 100_000, "test").unwrap();
    jobs::submit(&root, &spec(&root, "receipt", 0.25)).unwrap();
    let dir = root.join("running/receipt");
    fs::rename(root.join("queue/receipt"), &dir).unwrap();
    jobs::prepare_attempt(&dir, "receipt").unwrap();
    let attempt = jobs::read_attempt(&dir).unwrap();
    let job = jobs::read_spec(&dir).unwrap();
    let identity = lab_runner::process::Identity {
        pid: u32::MAX,
        pgid: u32::MAX,
        started: "absent fixture".into(),
        boot: "unverified-boot-fixture".into(),
        host: lab_runner::process::host_id().unwrap(),
        token: attempt.process_token.clone(),
        supervisor_started: "absent fixture".into(),
    };
    fs::write(
        dir.join("process.json"),
        serde_json::to_vec(&identity).unwrap(),
    )
    .unwrap();
    // Simulate crash after positive absence, receipt and charge, before rename.
    let exit = json!({"schema":jobs::EXIT_SCHEMA,"id":"receipt","lab":job.lab,"wall_s":job.wall_s,
        "spec_sha256":lab_runner::coord::digest(&serde_json::to_vec(&job).unwrap()),
        "attempt_id":attempt.attempt_id,"host":lab_runner::process::host_id().unwrap(),
        "process_state":"confirmed_stopped","outcome":"completed","elapsed_ms":12,"exit_status":0});
    fs::write(dir.join("exit.json"), serde_json::to_vec(&exit).unwrap()).unwrap();
    ledger::record_attempt_charge(
        &ledger,
        &job.lab,
        &job.id,
        &attempt.attempt_id,
        12,
        job.wall_s,
        "completed",
    )
    .unwrap();
    jobs::replay_finalization(&root, &dir, &ledger).unwrap();
    assert!(root.join("done/receipt/exit.json").exists());
    assert_eq!(ledger::rebuild(&ledger).unwrap().cumulative_ms, 12);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn stopped_reconciliation_preserves_unknown_and_charges_supplement_once() {
    let root = fixture("reconcile");
    let ledger = root.join("ledger");
    fs::create_dir(&ledger).unwrap();
    ledger::initialize_empty(&ledger, 100_000, "test").unwrap();
    jobs::submit(&root, &spec(&root, "reconcile", 0.25)).unwrap();
    let dir = root.join("running/reconcile");
    jobs::durable_rename(&root.join("queue/reconcile"), &dir).unwrap();
    let attempt = jobs::prepare_attempt(&dir, "reconcile").unwrap();
    let identity = lab_runner::process::Identity {
        pid: u32::MAX,
        pgid: u32::MAX,
        started: "absent process table fixture".into(),
        boot: "legacy-boot-time-is-not-reboot-proof".into(),
        host: lab_runner::process::host_id().unwrap(),
        token: attempt.process_token,
        supervisor_started: "fixture".into(),
    };
    fs::write(
        dir.join("process.json"),
        serde_json::to_vec(&identity).unwrap(),
    )
    .unwrap();
    let job = jobs::read_spec(&dir).unwrap();
    jobs::finalize(
        &root,
        &job,
        &jobs::Finalization {
            outcome: "unknown".into(),
            exit_status: None,
            reason: "unknown observation; later process-table absence is separate".into(),
            peak_rss_kib: 0,
            started_utc: attempt.started_utc,
            elapsed_ms: 12,
            measurement: ledger::Measurement::Measured,
        },
        &ledger,
    )
    .unwrap();
    let original_path = root.join("done/reconcile/exit.json");
    let original = fs::read(dir.join("exit.json")).unwrap();
    assert!(dir.is_dir());
    assert!(!original_path.exists());
    // Preserved legacy fixture: no recorded host in its process identity.
    // Its original exit does bind this host; no boot UUID is backfilled.
    let mut legacy = serde_json::to_value(&identity).unwrap();
    legacy.as_object_mut().unwrap().remove("host");
    fs::write(
        dir.join("process.json"),
        serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    let receipt = jobs::reconcile_stopped(&root, "reconcile", &ledger).unwrap();
    let bytes = fs::read(&receipt).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["scientific_outcome"], "unknown");
    assert_eq!(value["process_state"], "confirmed_stopped");
    assert_eq!(fs::read(&original_path).unwrap(), original);
    let charged = ledger::rebuild(&ledger).unwrap().cumulative_ms;
    assert!(charged >= job.reserved_ms().unwrap());
    jobs::reconcile_stopped(&root, "reconcile", &ledger).unwrap();
    assert_eq!(ledger::rebuild(&ledger).unwrap().cumulative_ms, charged);
    assert_eq!(fs::read(&receipt).unwrap(), bytes);
    // Crash after supplemental debit but before publishing its receipt.
    let intent = receipt.with_file_name("reconciliation-intent.json");
    fs::rename(&receipt, &intent).unwrap();
    jobs::reconcile_stopped(&root, "reconcile", &ledger).unwrap();
    assert_eq!(ledger::rebuild(&ledger).unwrap().cumulative_ms, charged);
    assert_eq!(fs::read(&receipt).unwrap(), bytes);
    // The same persisted intent before debit, in an isolated original-only
    // ledger, must apply exactly that supplement once without recomputing time.
    let before_debit = root.join("before-debit-ledger");
    fs::create_dir(&before_debit).unwrap();
    ledger::initialize_empty(&before_debit, 100_000, "before-debit fixture").unwrap();
    for entry in fs::read_dir(&ledger).unwrap().map(|e| e.unwrap()) {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with("charge-v2-")
        {
            let record: ledger::ChargeRecord =
                serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
            if record.attempt_id == attempt.attempt_id {
                fs::copy(entry.path(), before_debit.join(entry.file_name())).unwrap();
            }
        }
    }
    assert_eq!(ledger::rebuild(&before_debit).unwrap().cumulative_ms, 12);
    fs::rename(&receipt, &intent).unwrap();
    jobs::reconcile_stopped(&root, "reconcile", &before_debit).unwrap();
    assert_eq!(
        ledger::rebuild(&before_debit).unwrap().cumulative_ms,
        charged
    );
    jobs::reconcile_stopped(&root, "reconcile", &before_debit).unwrap();
    assert_eq!(
        ledger::rebuild(&before_debit).unwrap().cumulative_ms,
        charged
    );
    assert_eq!(fs::read(&receipt).unwrap(), bytes);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn running_reconciliation_preserves_estimated_measurement_in_charge() {
    let root = fixture("running-reconcile-estimate");
    let ledger = root.join("ledger");
    fs::create_dir(&ledger).unwrap();
    ledger::initialize_empty(&ledger, 100_000, "test").unwrap();
    jobs::submit(&root, &spec(&root, "running-estimate", 0.25)).unwrap();
    let dir = root.join("running/running-estimate");
    jobs::durable_rename(&root.join("queue/running-estimate"), &dir).unwrap();
    let attempt = jobs::prepare_attempt(&dir, "running-estimate").unwrap();
    let identity = lab_runner::process::Identity {
        pid: u32::MAX,
        pgid: u32::MAX,
        started: "absent process table fixture".into(),
        boot: "legacy-boot-time-is-not-reboot-proof".into(),
        host: lab_runner::process::host_id().unwrap(),
        token: attempt.process_token,
        supervisor_started: "fixture".into(),
    };
    fs::write(
        dir.join("process.json"),
        serde_json::to_vec(&identity).unwrap(),
    )
    .unwrap();
    jobs::reconcile_stopped(&root, "running-estimate", &ledger).unwrap();
    let exit: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("done/running-estimate/exit.json")).unwrap())
            .unwrap();
    assert_eq!(exit["outcome"], "unknown");
    assert_eq!(exit["measurement"], "estimated");
    let mut charges = 0;
    for entry in fs::read_dir(&ledger).unwrap().map(|e| e.unwrap()) {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with("charge-v2-")
        {
            let charge: ledger::ChargeRecord =
                serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
            assert_eq!(
                charge.accounting.unwrap().measurement,
                ledger::Measurement::Estimated
            );
            charges += 1;
        }
    }
    assert_eq!(charges, 2);
    fs::remove_dir_all(root).unwrap();
}
