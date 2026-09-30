//! End-to-end tests against temporary queue roots and temporary ledgers.
//! Nothing here touches the real runner root, the real ledger, or launchd.

use lab_runner::daemon::{self, DaemonConfig};
use lab_runner::{jobs, ledger};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lab-runner-test-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_spec(dir: &Path, spec: &Value) -> PathBuf {
    let path = dir.join("spec.json");
    fs::write(&path, serde_json::to_vec_pretty(spec).unwrap()).unwrap();
    path
}

fn base_spec(id: &str, cwd: &Path, argv: Value) -> Value {
    json!({
        "schema": "uor-r4.lab-runner-job/1",
        "id": id,
        "lab": "lab-test",
        "cwd": cwd,
        "argv": argv,
        "threads": 1,
        "rss_gib": 0.25,
        "gpu": false,
        "wall_s": 30,
        "kill_criterion": {"kind": "wall", "value": ""},
        "exclusive": false
    })
}

/// Run the daemon on a background thread until `predicate` holds, then stop
/// it via the stop file and join. Returns false on timeout.
fn run_daemon_until(
    root: &Path,
    ledger: &Path,
    predicate: impl Fn() -> bool,
    timeout: Duration,
) -> bool {
    if !ledger.join("ledger-baseline.json").is_file() {
        ledger::initialize_empty(ledger, 10_000_000, "isolated runner test fixture").unwrap();
    }
    let stop = root.join("STOP");
    let config = DaemonConfig {
        root: root.to_path_buf(),
        ledger_dir: ledger.to_path_buf(),
        poll_interval: Duration::from_millis(100),
        monitor_interval: Duration::from_millis(200),
        stop_file: Some(stop.clone()),
        require_host_policy: false,
    };
    let handle = std::thread::spawn(move || daemon::run(&config));
    let deadline = Instant::now() + timeout;
    let mut ok = false;
    while Instant::now() < deadline {
        if predicate() {
            ok = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    fs::write(&stop, b"stop\n").unwrap();
    handle.join().unwrap().unwrap();
    ok
}

fn exit_json(root: &Path, id: &str) -> Value {
    let path = root.join("done").join(id).join("exit.json");
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn charge_files(ledger: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(ledger)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .map(|n| n.to_string_lossy().starts_with("charge-"))
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    files
}

#[test]
fn cli_round_trip_submit_run_complete_and_charge() {
    let root = tempdir("roundtrip-root");
    let ledger_dir = tempdir("roundtrip-ledger");
    let work = tempdir("roundtrip-cwd");
    ledger::initialize_empty(&ledger_dir, 10_000_000, "isolated runner test fixture").unwrap();

    let spec = base_spec(
        "roundtrip-1",
        &work,
        json!(["sh", "-c", "echo ok; sleep 2"]),
    );
    let spec_path = write_spec(&work, &spec);

    let binary = env!("CARGO_BIN_EXE_lab-runner");
    let output = Command::new(binary)
        .arg("--root")
        .arg(&root)
        .arg("--ledger-dir")
        .arg(&ledger_dir)
        .arg("submit")
        .arg(&spec_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "submit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "roundtrip-1"
    );
    assert!(root.join("queue/roundtrip-1/spec.json").is_file());

    // A duplicate id is rejected.
    let output = Command::new(binary)
        .arg("--root")
        .arg(&root)
        .arg("--ledger-dir")
        .arg(&ledger_dir)
        .arg("submit")
        .arg(&spec_path)
        .output()
        .unwrap();
    assert!(!output.status.success());

    let done = root.join("done/roundtrip-1");
    assert!(run_daemon_until(
        &root,
        &ledger_dir,
        || done.is_dir(),
        Duration::from_secs(30)
    ));

    let exit = exit_json(&root, "roundtrip-1");
    assert_eq!(exit["schema"], "uor-r4.lab-runner-exit/1");
    assert_eq!(exit["outcome"], "completed");
    assert_eq!(exit["exit_status"], 0);
    assert!(exit["elapsed_ms"].as_u64().unwrap() >= 1500);
    let stdout = fs::read_to_string(done.join("stdout.log")).unwrap();
    assert!(stdout.contains("ok"));

    let charges = charge_files(&ledger_dir);
    assert_eq!(charges.len(), 1);
    let charge: Value = serde_json::from_slice(&fs::read(&charges[0]).unwrap()).unwrap();
    assert_eq!(charge["schema"], ledger::CHARGE_SCHEMA);
    assert_eq!(charge["job_id"], "roundtrip-1");
    assert_eq!(charge["runner"], "lab-runner");
    assert_eq!(charge["outcome"], "completed");
    let state: Value =
        serde_json::from_slice(&fs::read(ledger_dir.join("model-time.json")).unwrap()).unwrap();
    assert_eq!(
        state["cumulative_ms"].as_u64().unwrap(),
        charge["charged_ms"].as_u64().unwrap()
    );

    // status works without the daemon running.
    let output = Command::new(binary)
        .arg("--root")
        .arg(&root)
        .arg("--ledger-dir")
        .arg(&ledger_dir)
        .args(["status", "roundtrip-1"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let status: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(status["stage"], "done");

    // tail prints the captured logs.
    let output = Command::new(binary)
        .arg("--root")
        .arg(&root)
        .arg("--ledger-dir")
        .arg(&ledger_dir)
        .args(["tail", "roundtrip-1"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("ok"));

    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&ledger_dir).unwrap();
    fs::remove_dir_all(&work).unwrap();
}

#[test]
fn wall_time_kill_terminates_a_sleeping_job() {
    let root = tempdir("wall-root");
    let ledger_dir = tempdir("wall-ledger");
    let work = tempdir("wall-cwd");
    let mut spec = base_spec("wall-1", &work, json!(["sh", "-c", "sleep 30"]));
    spec["wall_s"] = json!(1);
    let spec_path = write_spec(&work, &spec);
    jobs::submit(&root, &spec_path).unwrap();

    let done = root.join("done/wall-1");
    let started = Instant::now();
    assert!(run_daemon_until(
        &root,
        &ledger_dir,
        || done.is_dir(),
        Duration::from_secs(30)
    ));
    let exit = exit_json(&root, "wall-1");
    assert_eq!(exit["outcome"], "wall_killed");
    let elapsed = exit["elapsed_ms"].as_u64().unwrap();
    assert!(
        (500..=10_000).contains(&elapsed),
        "elapsed {elapsed} ms should be near the 1 s wall"
    );
    assert!(started.elapsed() < Duration::from_secs(20));
    assert_eq!(charge_files(&ledger_dir).len(), 1);

    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&ledger_dir).unwrap();
    fs::remove_dir_all(&work).unwrap();
}

#[test]
fn log_match_criterion_kills_on_literal_substring() {
    let root = tempdir("log-root");
    let ledger_dir = tempdir("log-ledger");
    let work = tempdir("log-cwd");
    let mut spec = base_spec(
        "log-1",
        &work,
        json!(["sh", "-c", "echo before; echo FATAL-MARKER; sleep 30"]),
    );
    spec["wall_s"] = json!(60);
    spec["kill_criterion"] = json!({"kind": "log_match", "value": "FATAL-MARKER"});
    let spec_path = write_spec(&work, &spec);
    jobs::submit(&root, &spec_path).unwrap();

    let done = root.join("done/log-1");
    assert!(run_daemon_until(
        &root,
        &ledger_dir,
        || done.is_dir(),
        Duration::from_secs(30)
    ));
    let exit = exit_json(&root, "log-1");
    assert_eq!(exit["outcome"], "criterion_killed");
    assert!(exit["elapsed_ms"].as_u64().unwrap() < 20_000);

    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&ledger_dir).unwrap();
    fs::remove_dir_all(&work).unwrap();
}

#[test]
fn daemon_restart_preserves_legacy_unknown_without_signalling() {
    let root = tempdir("crash-root");
    let ledger_dir = tempdir("crash-ledger");
    let work = tempdir("crash-cwd");
    // Plant a running/ entry whose pid is dead: spawn a trivial process and
    // reap it, then reuse its (now dead) pid.
    let mut child = Command::new("true").spawn().unwrap();
    let dead_pid = child.id();
    child.wait().unwrap();
    assert!(!jobs::pid_alive(dead_pid));

    let spec = base_spec("crash-1", &work, json!(["sh", "-c", "sleep 5"]));
    let running = root.join("running/crash-1");
    fs::create_dir_all(&running).unwrap();
    fs::write(
        running.join("spec.json"),
        serde_json::to_vec_pretty(&spec).unwrap(),
    )
    .unwrap();
    fs::write(running.join("pid"), format!("{dead_pid}\n")).unwrap();
    fs::write(running.join("started_utc"), "2026-01-01T00:00:00Z\n").unwrap();
    fs::write(running.join("started_ms"), b"1\n").unwrap();

    let unknown = root.join("running/crash-1/recovery-required.json");
    assert!(run_daemon_until(
        &root,
        &ledger_dir,
        || unknown.is_file(),
        Duration::from_secs(15)
    ));
    assert!(root.join("admissions-held.json").is_file());
    assert!(root.join("running/crash-1/spec.json").is_file());
    assert!(charge_files(&ledger_dir).is_empty());

    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&ledger_dir).unwrap();
    fs::remove_dir_all(&work).unwrap();
}

#[test]
fn cancel_finalizes_a_running_job() {
    let root = tempdir("cancel-root");
    let ledger_dir = tempdir("cancel-ledger");
    let work = tempdir("cancel-cwd");
    let spec = base_spec("cancel-1", &work, json!(["sh", "-c", "sleep 60"]));
    let spec_path = write_spec(&work, &spec);
    jobs::submit(&root, &spec_path).unwrap();

    ledger::initialize_empty(&ledger_dir, 10_000_000, "isolated runner test fixture").unwrap();
    let running = root.join("running/cancel-1");
    let stop = root.join("STOP");
    let config = DaemonConfig {
        root: root.clone(),
        ledger_dir: ledger_dir.clone(),
        poll_interval: Duration::from_millis(100),
        monitor_interval: Duration::from_millis(200),
        stop_file: Some(stop.clone()),
        require_host_policy: false,
    };
    let handle = std::thread::spawn(move || daemon::run(&config));
    let deadline = Instant::now() + Duration::from_secs(15);
    while !running.join("pid").is_file() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(running.join("pid").is_file(), "job never started");

    jobs::cancel(&root, "cancel-1", &ledger_dir).unwrap();
    fs::write(&stop, b"stop\n").unwrap();
    handle.join().unwrap().unwrap();

    let exit = exit_json(&root, "cancel-1");
    assert_eq!(exit["outcome"], "cancelled");
    assert_eq!(charge_files(&ledger_dir).len(), 1);

    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&ledger_dir).unwrap();
    fs::remove_dir_all(&work).unwrap();
}

#[test]
fn cancel_removes_a_queued_job() {
    let root = tempdir("cancelq-root");
    let ledger_dir = tempdir("cancelq-ledger");
    let work = tempdir("cancelq-cwd");
    let spec = base_spec("cancel-q", &work, json!(["true"]));
    let spec_path = write_spec(&work, &spec);
    jobs::submit(&root, &spec_path).unwrap();
    assert!(root.join("queue/cancel-q").is_dir());
    jobs::cancel(&root, "cancel-q", &ledger_dir).unwrap();
    assert!(!root.join("queue/cancel-q").exists());
    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&ledger_dir).unwrap();
    fs::remove_dir_all(&work).unwrap();
}
