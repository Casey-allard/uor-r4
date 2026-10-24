//! The loop PR 10 supervisor tests: the planner is pure and time-injected, and the process layer is
//! exercised on real short-lived shells. The `loop-drive` gate runs this file; its planted fault must
//! turn it red with the item's own sentence: **a fake stalled client past 2 hours is closed and
//! carded**, while a fake progressing one continues with a note.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Command;

use uor_r4_harness::loopdrive::{
    check_in, classify, close_stalled_session, commit_and_push_wip, compose_prompt, launch_argv,
    model_in_launch, note_body, note_issue, pm_card_text, post_note_with, read_record,
    run_sessions, session_alive, start_session, terminate_session, watch_lease_until_renew_fails,
    watch_until_exit, CheckIn, DriveHooks, DriveSpec, Next, Observation, Progress, SessionRecord,
    CHECK_IN_SECONDS,
};

fn observation(head: &str, commits: u64, gate_runs: u64, changed_paths: usize) -> Observation {
    Observation {
        head: head.to_string(),
        commits,
        gate_runs,
        changed_paths,
    }
}

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("uor-loop-drive-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("temp root");
    root
}

const GIT_VARS: [&str; 8] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_NAMESPACE",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
];

fn git_in(root: &Path, args: &[&str]) {
    assert!(
        root.starts_with(std::env::temp_dir()),
        "a test repository must stay under the temp dir: {}",
        root.display()
    );
    let mut command = Command::new("git");
    command.args(args).current_dir(root);
    for name in GIT_VARS {
        command.env_remove(name);
    }
    let status = command.status().expect("git runs");
    assert!(status.success(), "git {args:?} failed");
}

fn init_repo(name: &str) -> PathBuf {
    let root = temp_root(name);
    git_in(&root, &["init", "-q", "-b", "main"]);
    git_in(&root, &["config", "user.email", "drive@test"]);
    git_in(&root, &["config", "user.name", "drive test"]);
    std::fs::write(root.join("initial.txt"), "initial\n").expect("write");
    std::fs::write(root.join(".gitignore"), "local/\n").expect("write");
    git_in(&root, &["add", "initial.txt", ".gitignore"]);
    git_in(&root, &["commit", "-q", "-m", "initial"]);
    root
}

#[test]
fn the_soft_check_in_continues_real_progress_and_closes_a_stall() {
    assert!(matches!(
        check_in(60, &Progress::Stalled, 1),
        CheckIn::Continue { .. }
    ));
    match check_in(CHECK_IN_SECONDS, &Progress::Progressing("new commits"), 0) {
        CheckIn::Continue { note } => assert!(note.contains("real progress"), "{note}"),
        other => panic!("a progressing session must continue with a note, got {other:?}"),
    }
    match check_in(CHECK_IN_SECONDS, &Progress::Stalled, 1) {
        CheckIn::Close { reason } => assert!(reason.starts_with("stalled:"), "{reason}"),
        other => panic!("a fake stalled client past 2 hours is closed and carded: {other:?}"),
    }
    match check_in(CHECK_IN_SECONDS, &Progress::Stalled, 2) {
        CheckIn::Close { reason } => assert!(reason.starts_with("spiraled:"), "{reason}"),
        other => panic!("a repeated stall is spiraled: {other:?}"),
    }
}

#[test]
fn progress_is_commits_gate_runs_or_edits_and_nothing_else() {
    let before = observation("aaaa", 10, 3, 0);
    assert_eq!(
        classify(&before, &observation("bbbb", 10, 3, 0)),
        Progress::Progressing("new HEAD")
    );
    assert_eq!(
        classify(&before, &observation("aaaa", 11, 3, 0)),
        Progress::Progressing("new commits")
    );
    assert_eq!(
        classify(&before, &observation("aaaa", 10, 4, 0)),
        Progress::Progressing("new gate runs")
    );
    assert_eq!(
        classify(&before, &observation("aaaa", 10, 3, 2)),
        Progress::Progressing("working tree changed")
    );
    assert_eq!(classify(&before, &before), Progress::Stalled);
}

#[test]
fn the_launch_command_gets_exactly_one_prompt_and_not_as_the_program() {
    let argv = vec!["dsh".to_string(), "{prompt}".to_string()];
    assert_eq!(
        launch_argv(&argv, "prompt text").unwrap(),
        vec!["dsh".to_string(), "prompt text".to_string()]
    );
    assert!(launch_argv(&["dsh".to_string()], "x").is_err());
    assert!(launch_argv(&["{prompt}".to_string(), "{prompt}".to_string()], "x").is_err());
    assert!(launch_argv(&["{prompt}".to_string(), "arg".to_string()], "x").is_err());
    let default = compose_prompt(None, "GO item infra-loop-pr10 - ready");
    assert!(
        default.contains("GO item infra-loop-pr10 - ready"),
        "{default}"
    );
    assert_eq!(compose_prompt(Some("mine"), "GO x"), "mine");
}

#[test]
fn a_real_short_session_is_tracked_and_its_record_read_back() {
    let root = temp_root("session");
    let log = root.join("log/session.log");
    let (mut child, record) = start_session(
        &root,
        "deepseek",
        "fake-client",
        "pinned-model-1",
        &["sh".to_string(), "-c".to_string(), "sleep 0.4".to_string()],
        "deadbeefdeadbeef",
        &log,
    )
    .expect("start a short session");
    assert_eq!(record.client, "fake-client");
    assert!(
        !record.started_lstart.is_empty(),
        "identity recorded: {record:?}"
    );
    assert!(log.exists(), "the session log is created");
    assert!(session_alive(&record), "the sleeping child is alive");
    let read = read_record(&root, "deepseek").expect("the record is on disk before the wait");
    assert_eq!(read.pid, record.pid);
    assert_eq!(read.model, "pinned-model-1");
    let _ = terminate_session(&record);
    let _ = child.wait();
    assert!(!session_alive(&record), "the recorded session is gone");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_supervisor_signals_only_its_recorded_session_and_waits_it_out() {
    let root = temp_root("terminate");
    let log = root.join("session.log");
    let (mut child, record) = start_session(
        &root,
        "deepseek",
        "fake-client",
        "pinned-model-1",
        &["sh".to_string(), "-c".to_string(), "sleep 30".to_string()],
        "deadbeefdeadbeef",
        &log,
    )
    .expect("start a long session");
    std::thread::sleep(std::time::Duration::from_millis(200));
    terminate_session(&record).expect("terminate the recorded session");
    let waited = child.wait().expect("wait");
    assert!(
        !waited.success(),
        "the long session is signalled, not left running"
    );
    assert!(!session_alive(&record), "the recorded process is gone");
    assert!(
        !terminate_session(&record).is_err(),
        "a dead session is not an error"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_watcher_outlives_the_run_and_never_cancels_it() {
    let root = temp_root("watch");
    let log = root.join("session.log");
    let (mut child, record) = start_session(
        &root,
        "deepseek",
        "fake-client",
        "pinned-model-1",
        &["sh".to_string(), "-c".to_string(), "sleep 30".to_string()],
        "deadbeefdeadbeef",
        &log,
    )
    .expect("start the watched session");
    std::thread::sleep(std::time::Duration::from_millis(200));
    let notes = watch_until_exit(&record, &["true".to_string()], 0, Some(2)).expect("watch");
    assert!(
        notes.iter().any(|note| note.contains("renewed")),
        "the watcher renews while the run is alive: {notes:?}"
    );
    assert!(
        session_alive(&record),
        "the watcher never cancels a healthy run"
    );
    terminate_session(&record).expect("terminate");
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_lease_watcher_outlives_the_session_and_stops_when_the_lease_ends() {
    let notes = watch_lease_until_renew_fails(&["true".to_string()], 0, Some(2)).expect("watch");
    assert!(
        notes.iter().any(|note| note.contains("renewed")),
        "the watcher renews a healthy lease: {notes:?}"
    );
    assert!(
        notes.iter().any(|note| note.contains("test bound")),
        "the bounded watch stops: {notes:?}"
    );
    let notes = watch_lease_until_renew_fails(&["false".to_string()], 0, Some(2)).expect("watch");
    assert!(
        notes.iter().any(|note| note.contains("lease ended")),
        "a failed renew ends the watch without cancelling: {notes:?}"
    );
}

#[test]
fn the_pm_card_names_the_lab_reason_head_and_needed_prompt() {
    let card = pm_card_text("deepseek", "stalled: no commits", "abc1234", 7);
    assert!(card.starts_with("[deepseek]"), "{card}");
    assert!(card.contains("stalled: no commits"), "{card}");
    assert!(card.contains("abc1234"), "{card}");
    assert!(card.contains("needs a prompt"), "{card}");
}

#[test]
fn a_closed_session_records_a_report_stub_and_a_pickup_from_git_state() {
    let root = temp_root("close");
    let record = SessionRecord {
        client: "fake-client".to_string(),
        model: "pinned-model-1".to_string(),
        pid: 1,
        pgid: 1,
        started_unix: 0,
        started_lstart: "identity".to_string(),
        nonce: "nonce".to_string(),
        prompt_sha256: "deadbeef".to_string(),
    };
    let directory = close_stalled_session(
        &root,
        "deepseek",
        &record,
        "stalled: no commits, gate runs or edits for 7200s of the 2-hour task",
    )
    .expect("close the stalled session");
    let stub = std::fs::read_to_string(directory.join("REPORT-stub.md")).expect("stub");
    assert!(stub.contains("stalled:"), "{stub}");
    assert!(stub.contains("no result is claimed"), "{stub}");
    let pickup = std::fs::read_to_string(directory.join("PICKUP.md")).expect("pickup");
    assert!(pickup.contains("loop next"), "{pickup}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_real_repository_reads_a_head_and_commit_count() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the checkout root");
    let observation = Observation::take(&root, "deepseek");
    assert!(observation.head.len() >= 7, "HEAD: {observation:?}");
    assert!(observation.commits > 0, "commits: {observation:?}");
}

/// The close records a run left behind, if any: `local/loop/drive/<lab>/close-*`.
fn close_dirs(root: &Path) -> Vec<PathBuf> {
    let directory = root.join("local/loop/drive/deepseek");
    let Ok(entries) = std::fs::read_dir(&directory) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("close-"))
        })
        .collect()
}

/// A fake clock that jumps 3,600 s on every read, so the 2-hour check-in happens within a few polls.
fn fake_clock() -> impl Fn() -> u64 {
    let jump = std::cell::Cell::new(0u64);
    move || {
        let seconds = jump.get();
        jump.set(seconds + 3_600);
        let real = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        real + seconds
    }
}

#[test]
fn the_fake_clock_closes_a_silent_client_and_cards_it() {
    let root = init_repo("silent");
    let launch = vec![
        "sh".to_string(),
        "-c".to_string(),
        "sleep 0.6".to_string(),
        "{prompt}".to_string(),
    ];
    let cards: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let decision = || -> std::result::Result<Next, String> {
        Ok(Next::Go("GO item fake - ready".to_string()))
    };
    let card = |reason: &str| {
        cards.borrow_mut().push(reason.to_string());
        Ok(())
    };
    let now = fake_clock();
    let pause = |_seconds: u64| std::thread::sleep(std::time::Duration::from_millis(15));
    let spec = DriveSpec {
        root: &root,
        lab: "deepseek",
        client: "fake-client",
        model: "pinned-model-1",
        launch: &launch,
        prompt: Some("fake session"),
        max_sessions: 1,
        order: None,
        exempt_lease: false,
        poll: 600,
    };
    let notice = |_note: &str| {};
    let hooks = DriveHooks {
        now: &now,
        pause: &pause,
        decision: &decision,
        card: &card,
        notice: &notice,
    };
    let code = run_sessions(&spec, &hooks).expect("the silent session runs");
    assert_eq!(code, 1, "the closed session reports a non-zero code");
    let cards = cards.into_inner();
    assert_eq!(cards.len(), 1, "exactly one card: {cards:?}");
    assert!(
        cards[0].starts_with("stalled:"),
        "a fake stalled client past 2 hours is closed and carded: {cards:?}"
    );
    assert!(
        !close_dirs(&root).is_empty(),
        "the close record is written from git state"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_fake_clock_continues_a_committing_client_with_a_note() {
    let root = init_repo("progress");
    let script = "echo more >> work.txt && git add work.txt && git commit -q -m wip && touch commit.done && sleep 0.6";
    let launch = vec![
        "sh".to_string(),
        "-c".to_string(),
        script.to_string(),
        "{prompt}".to_string(),
    ];
    let cards: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let decision = || -> std::result::Result<Next, String> {
        Ok(Next::Go("GO item fake - ready".to_string()))
    };
    let card = |reason: &str| {
        cards.borrow_mut().push(reason.to_string());
        Ok(())
    };
    // The clock only advances past the check-in after the fake client has committed, so the
    // test is deterministic: before the marker exists, real time is returned.
    let marker = root.join("commit.done");
    let jump = std::cell::Cell::new(0u64);
    let now = move || {
        let real = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        if !marker.exists() {
            return real;
        }
        // One check-in only: the clock stops at the first window, so the committing client is
        // seen as progressing and then exits before a second window could open.
        let seconds = (jump.get() + 3_600).min(7_200);
        jump.set(seconds);
        real + seconds
    };
    let pause = |_seconds: u64| std::thread::sleep(std::time::Duration::from_millis(60));
    let spec = DriveSpec {
        root: &root,
        lab: "deepseek",
        client: "fake-client",
        model: "pinned-model-1",
        launch: &launch,
        prompt: Some("fake session"),
        max_sessions: 1,
        order: None,
        exempt_lease: false,
        poll: 600,
    };
    let notice = |_note: &str| {};
    let hooks = DriveHooks {
        now: &now,
        pause: &pause,
        decision: &decision,
        card: &card,
        notice: &notice,
    };
    let code = run_sessions(&spec, &hooks).expect("the progressing session runs");
    assert_eq!(code, 0, "the progressing client exited cleanly");
    assert!(
        cards.into_inner().is_empty(),
        "a fake progressing client past 2 hours continues with a note, not a card"
    );
    assert!(
        close_dirs(&root).is_empty(),
        "a progressing session is not closed"
    );
    let _ = std::fs::remove_dir_all(&root);
}

fn drive_repo(name: &str) -> PathBuf {
    let root = init_repo(name);
    let order = root.join("docs/loop/tasks/deepseek-1/ORDER.md");
    std::fs::create_dir_all(order.parent().expect("folder")).expect("folder");
    std::fs::write(&order, "# ORDER: fixture\n").expect("order");
    git_in(&root, &["add", "docs"]);
    git_in(&root, &["commit", "-q", "-m", "order"]);
    root
}

fn git_out(root: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    command.args(args).current_dir(root);
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    let out = command.output().expect("git runs");
    String::from_utf8_lossy(&out.stdout).to_string()
}
#[test]
fn the_wip_commit_refuses_main_a_foreign_branch_and_a_foreign_folder() {
    let root = drive_repo("wip-refuse");
    let order = root.join("docs/loop/tasks/deepseek-1/ORDER.md");
    // On main: refused before any git write.
    let error = commit_and_push_wip(&root, "deepseek", Some(&order), "stalled").expect_err("main");
    assert!(
        error.to_string().contains("not a deepseek/ branch"),
        "{error}"
    );
    // A different lab branch: refused.
    git_in(&root, &["checkout", "-q", "-b", "kimi/other"]);
    let error = commit_and_push_wip(&root, "deepseek", Some(&order), "stalled")
        .expect_err("foreign branch");
    assert!(
        error.to_string().contains("not a deepseek/ branch"),
        "{error}"
    );
    git_in(&root, &["checkout", "-q", "-b", "deepseek/topic"]);
    // A task folder that is not <lab>-<n>: refused.
    let foreign = root.join("docs/loop/tasks/other-1/ORDER.md");
    std::fs::create_dir_all(foreign.parent().expect("folder")).expect("folder");
    std::fs::write(&foreign, "# ORDER: other\n").expect("order");
    let error = commit_and_push_wip(&root, "deepseek", Some(&foreign), "stalled")
        .expect_err("foreign folder");
    assert!(
        error.to_string().contains("not a deepseek task folder"),
        "{error}"
    );
    // No ORDER at all: refused rather than staging everything.
    let error = commit_and_push_wip(&root, "deepseek", None, "stalled").expect_err("no order");
    assert!(error.to_string().contains("unscoped"), "{error}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_wip_commit_stages_only_the_task_folder_and_carries_the_trailers() {
    let root = drive_repo("wip-commit");
    git_in(&root, &["checkout", "-q", "-b", "deepseek/topic"]);
    let order = root.join("docs/loop/tasks/deepseek-1/ORDER.md");
    std::fs::write(
        root.join("docs/loop/tasks/deepseek-1/REPORT.md"),
        "# REPORT\n",
    )
    .expect("write");
    std::fs::write(root.join("unrelated.txt"), "do not commit\n").expect("write");
    // No origin exists, so the push fails after the commit; the commit itself is what this test reads.
    let error = commit_and_push_wip(&root, "deepseek", Some(&order), "stalled: test")
        .expect_err("push without origin");
    assert!(error.to_string().contains("push origin"), "{error}");
    let message = git_out(&root, &["log", "-1", "--format=%B"]);
    assert!(message.contains("Lab: deepseek"), "{message}");
    assert!(
        message.contains("Order: docs/loop/tasks/deepseek-1/ORDER.md"),
        "{message}"
    );
    let changed = git_out(&root, &["show", "--name-only", "--format=", "HEAD"]);
    assert!(changed.contains("deepseek-1/REPORT.md"), "{changed}");
    assert!(
        !changed.contains("unrelated.txt"),
        "scoped, not -A: {changed}"
    );
    let status = git_out(&root, &["status", "--porcelain"]);
    assert!(
        status.contains("?? unrelated.txt"),
        "the out-of-scope file stays untracked"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_productive_then_silent_client_is_closed_at_the_second_window() {
    let root = init_repo("two-window");
    let script = "echo more >> work.txt && git add work.txt && git commit -q -m wip && touch commit.done && sleep 1.4";
    let launch = vec![
        "sh".to_string(),
        "-c".to_string(),
        script.to_string(),
        "{prompt}".to_string(),
    ];
    let cards: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let decision = || -> std::result::Result<Next, String> {
        Ok(Next::Go("GO item fake - ready".to_string()))
    };
    let card = |reason: &str| {
        cards.borrow_mut().push(reason.to_string());
        Ok(())
    };
    let marker = root.join("commit.done");
    let jump = std::cell::Cell::new(0u64);
    let now = move || {
        let real = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        if !marker.exists() {
            return real;
        }
        let seconds = jump.get();
        jump.set(seconds + 3_600);
        real + seconds
    };
    let pause = |_seconds: u64| std::thread::sleep(std::time::Duration::from_millis(15));
    let spec = DriveSpec {
        root: &root,
        lab: "deepseek",
        client: "fake-client",
        model: "pinned-model-1",
        launch: &launch,
        prompt: Some("fake session"),
        max_sessions: 1,
        order: None,
        exempt_lease: false,
        poll: 600,
    };
    let notice = |_note: &str| {};
    let hooks = DriveHooks {
        now: &now,
        pause: &pause,
        decision: &decision,
        card: &card,
        notice: &notice,
    };
    let code = run_sessions(&spec, &hooks).expect("the two-window session runs");
    let cards = cards.into_inner();
    assert_eq!(
        cards.len(),
        1,
        "closed only at the second window: {cards:?}"
    );
    assert!(
        cards[0].starts_with("stalled:"),
        "a fake stalled client past 2 hours is closed and carded: {cards:?}"
    );
    assert!(code == 1, "the close reports a non-zero code");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_leased_session_is_never_closed_and_notes_exactly_once_per_window() {
    // Deterministic: the fake clock steps 600 s per read up to the 30-poll horizon (18,000 s = 2.5
    // two-hour windows, so a 7,200 s window opens at read 12 and 14,400 s at read 24), and the stop
    // sentinel lives under `local/loop/drive/` (ignored by `Observation::take`) with a 40 ms child
    // teardown delay so parallel-load reap latency never opens a spurious third window or pollutes
    // `git status --porcelain`. With the `next_check` advance in `loopdrive.rs` removed, every poll
    // from read 12 onward notes and the exact count of 2 fails.
    let root = init_repo("lease-exempt");
    let launch = vec![
        "sh".to_string(),
        "-c".to_string(),
        "while [ ! -f local/loop/drive/stop ]; do sleep 0.01; done; sleep 0.04".to_string(),
        "{prompt}".to_string(),
    ];
    let cards: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let notices: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let decision = || -> std::result::Result<Next, String> {
        Ok(Next::Go("GO item lease - ready".to_string()))
    };
    let card = |reason: &str| {
        cards.borrow_mut().push(reason.to_string());
        Ok(())
    };
    let notice = |note: &str| notices.borrow_mut().push(note.to_string());
    let jump = std::cell::Cell::new(0u64);
    let now = move || {
        let real = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let seconds = jump.get();
        jump.set((seconds + 600).min(18_000));
        real + seconds
    };
    let polls = std::cell::Cell::new(0u32);
    let stop = root.join("local/loop/drive/stop");
    let pause = |_seconds: u64| {
        let count = polls.get() + 1;
        polls.set(count);
        if count >= 30 {
            let _ = std::fs::write(&stop, "stop\n");
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    };
    let spec = DriveSpec {
        root: &root,
        lab: "deepseek",
        client: "fake-client",
        model: "pinned-model-1",
        launch: &launch,
        prompt: Some("leased session"),
        max_sessions: 1,
        order: None,
        exempt_lease: true,
        poll: 600,
    };
    let hooks = DriveHooks {
        now: &now,
        pause: &pause,
        decision: &decision,
        card: &card,
        notice: &notice,
    };
    let code = run_sessions(&spec, &hooks).expect("the leased session runs");
    assert_eq!(code, 0, "a leased session is allowed to finish");
    assert!(
        cards.into_inner().is_empty(),
        "a leased session is never carded"
    );
    let notices = notices.into_inner();
    assert!(
        notices.iter().all(|note| note.contains("close skipped")),
        "every note is the lease skip: {notices:?}"
    );
    // The clock reaches 7,200 s at read 12 and 14,400 s at read 24; the run stops at 30 polls, so
    // exactly two windows open. Without advancing next_check (the removed lines) every poll after
    // read 12 notes, so this exact count is the planted assertion.
    assert_eq!(
        notices.len(),
        2,
        "one skip note per two-hour window, not one per poll: {notices:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A fake `loop next` that answers from a script, one answer per ask, and counts the asks.
fn scripted_next(
    answers: Vec<Next>,
) -> (
    impl Fn() -> std::result::Result<Next, String>,
    std::rc::Rc<std::cell::Cell<usize>>,
) {
    let asked = std::rc::Rc::new(std::cell::Cell::new(0usize));
    let counter = asked.clone();
    let next = move || {
        let index = counter.get();
        counter.set(index + 1);
        Ok(answers.get(index).cloned().unwrap_or(Next::Stop))
    };
    (next, asked)
}

fn sessions_started(root: &Path) -> usize {
    std::fs::read_dir(root.join("local/loop/drive/deepseek"))
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("session-"))
                .count()
        })
        .unwrap_or(0)
}

#[test]
fn a_wait_pauses_by_the_poll_and_asks_again_then_stop_ends_cleanly() {
    let root = init_repo("wait-stop");
    let launch = vec![
        "sh".to_string(),
        "-c".to_string(),
        "exit 0".to_string(),
        "{prompt}".to_string(),
    ];
    let (decision, asked) = scripted_next(vec![
        Next::Wait("nothing claimable".to_string()),
        Next::Stop,
    ]);
    let pauses: RefCell<Vec<u64>> = RefCell::new(Vec::new());
    let pause = |seconds: u64| pauses.borrow_mut().push(seconds);
    let now = || 1_000_000u64;
    let card = |_reason: &str| Ok(());
    let notice = |_note: &str| {};
    let spec = DriveSpec {
        root: &root,
        lab: "deepseek",
        client: "fake-client",
        model: "pinned-model-1",
        launch: &launch,
        prompt: Some("fake session"),
        max_sessions: 1,
        order: None,
        exempt_lease: false,
        poll: 37,
    };
    let hooks = DriveHooks {
        now: &now,
        pause: &pause,
        decision: &decision,
        card: &card,
        notice: &notice,
    };
    let code = run_sessions(&spec, &hooks).expect("WAIT then STOP is a clean run");
    assert_eq!(code, 0, "STOP after a WAIT ends the drive with 0");
    assert_eq!(
        pauses.into_inner(),
        vec![37],
        "one pause, by the poll value"
    );
    assert_eq!(asked.get(), 2, "loop next is asked again after the WAIT");
    assert_eq!(sessions_started(&root), 0, "a WAIT starts no session");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_wait_then_go_starts_exactly_one_session() {
    let root = init_repo("wait-go");
    let launch = vec![
        "sh".to_string(),
        "-c".to_string(),
        "exit 0".to_string(),
        "{prompt}".to_string(),
    ];
    let (decision, asked) = scripted_next(vec![
        Next::Wait("nothing claimable".to_string()),
        Next::Go("GO item fake - ready".to_string()),
    ]);
    let pauses: RefCell<Vec<u64>> = RefCell::new(Vec::new());
    let pause = |seconds: u64| {
        pauses.borrow_mut().push(seconds);
        if seconds != 37 {
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    };
    let now = || {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    };
    let card = |_reason: &str| Ok(());
    let notice = |_note: &str| {};
    let spec = DriveSpec {
        root: &root,
        lab: "deepseek",
        client: "fake-client",
        model: "pinned-model-1",
        launch: &launch,
        prompt: Some("fake session"),
        max_sessions: 1,
        order: None,
        exempt_lease: false,
        poll: 37,
    };
    let hooks = DriveHooks {
        now: &now,
        pause: &pause,
        decision: &decision,
        card: &card,
        notice: &notice,
    };
    let code = run_sessions(&spec, &hooks).expect("WAIT then GO runs one session");
    assert_eq!(code, 0, "the fake session exited cleanly");
    assert_eq!(asked.get(), 2, "loop next is asked again after the WAIT");
    assert_eq!(
        pauses.into_inner().first(),
        Some(&37),
        "the WAIT paused by the poll value first"
    );
    assert_eq!(
        sessions_started(&root),
        1,
        "exactly one session after the GO"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_check_in_note_is_tagged_and_carries_no_claim() {
    let body = note_body("deepseek", Some("#77"), "continues: 3 commits");
    assert!(
        body.starts_with("[deepseek] drive check-in (soft 2-hour, D53): continues: 3 commits"),
        "{body}"
    );
    assert!(body.contains("- session target: #77"), "{body}");
    assert!(body.contains("No result is claimed"), "{body}");
}

#[test]
fn post_note_with_pipes_the_body_to_gh_issue_comment() {
    use std::os::unix::fs::PermissionsExt;
    let root = temp_root("note-gh");
    let args_path = root.join("gh-args.txt");
    let body_path = root.join("gh-body.md");
    let script = root.join("fake-gh");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\necho \"$@\" > {args}\nprev=\"\"\nfor a in \"$@\"; do\n  if [ \"$prev\" = \"--body-file\" ]; then cp \"$a\" {body}; fi\n  prev=\"$a\"\ndone\n",
            args = args_path.display(),
            body = body_path.display(),
        ),
    )
    .expect("write fake gh");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    post_note_with(
        &root,
        &script,
        "deepseek",
        1234,
        Some("#77"),
        "continues: 3 commits",
    )
    .expect("the note posts");
    let args = std::fs::read_to_string(&args_path).expect("fake gh recorded its argv");
    assert!(args.contains("issue comment 1234 --body-file"), "{args}");
    let body = std::fs::read_to_string(&body_path).expect("fake gh recorded the body file");
    assert!(body.contains("[deepseek] drive check-in"), "{body}");
    assert!(body.contains("session target: #77"), "{body}");
    assert!(body.contains("No result is claimed"), "{body}");
    let drive = root.join("local/loop/drive/deepseek");
    let kept: Vec<PathBuf> = std::fs::read_dir(&drive)
        .expect("the drive directory exists")
        .map(|entry| entry.expect("a note entry").path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("md"))
        .collect();
    assert_eq!(kept.len(), 1, "one note file is kept: {kept:?}");
    let kept_body = std::fs::read_to_string(&kept[0]).expect("read the kept note");
    assert!(
        kept_body.contains("[deepseek] drive check-in"),
        "{kept_body}"
    );
    assert!(kept_body.contains("session target: #77"), "{kept_body}");
    // A second note in the same second gets its own file.
    post_note_with(&root, &script, "deepseek", 1234, None, "second note").expect("second note");
    let now = std::fs::read_dir(&drive)
        .expect("the drive directory exists")
        .count();
    assert_eq!(now, 2, "two notes in one second keep two files");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_model_pin_matches_whole_arguments_only() {
    let launch = vec![
        "dsh".to_string(),
        "headless".to_string(),
        "--model".to_string(),
        "deepseek-flash".to_string(),
        "{prompt}".to_string(),
    ];
    assert!(model_in_launch("deepseek-flash", &launch));
    // Prefix, placeholder and empty are not the model.
    assert!(!model_in_launch("deepseek", &launch));
    assert!(!model_in_launch("prompt", &launch));
    assert!(!model_in_launch("", &launch));
    assert!(!model_in_launch("claude-haiku", &launch));
    // The equals spellings and a bare argument count.
    assert!(model_in_launch(
        "deepseek-flash",
        &["dsh".to_string(), "--model=deepseek-flash".to_string()]
    ));
    assert!(model_in_launch(
        "deepseek-flash",
        &["kimi".to_string(), "-m=deepseek-flash".to_string()]
    ));
    assert!(model_in_launch(
        "deepseek-flash",
        &[
            "agy".to_string(),
            "-m".to_string(),
            "deepseek-flash".to_string()
        ]
    ));
}

#[test]
fn a_check_in_note_targets_the_named_pr_else_the_tracker() {
    assert_eq!(note_issue(Some("#1234")), 1234);
    assert_eq!(note_issue(Some("#0")), 0);
    assert_eq!(note_issue(Some("item infra-loop-pr10")), 2028);
    assert_eq!(note_issue(None), 2028);
}

#[test]
fn a_supervisor_stop_cards_the_pm_for_the_next_prompt() {
    let root = init_repo("stop-card");
    let launch = vec!["sh".to_string(), "-c".to_string(), "exit 0".to_string()];
    let (decision, _asked) = scripted_next(vec![Next::Stop]);
    let cards: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let card = |reason: &str| {
        cards.borrow_mut().push(reason.to_string());
        Ok(())
    };
    let now = || 1_000_000u64;
    let pause = |_seconds: u64| {};
    let notice = |_note: &str| {};
    let spec = DriveSpec {
        root: &root,
        lab: "deepseek",
        client: "fake-client",
        model: "pinned-model-1",
        launch: &launch,
        prompt: Some("fake session"),
        max_sessions: 1,
        order: None,
        exempt_lease: false,
        poll: 37,
    };
    let hooks = DriveHooks {
        now: &now,
        pause: &pause,
        decision: &decision,
        card: &card,
        notice: &notice,
    };
    let code = run_sessions(&spec, &hooks).expect("a STOP is a clean supervisor stop");
    assert_eq!(code, 0);
    let cards = cards.into_inner();
    assert_eq!(cards.len(), 1, "a STOP cards the PM once: {cards:?}");
    assert!(
        cards[0].starts_with("STOP: loop next asked the supervisor to stop"),
        "{cards:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn reaching_an_explicit_multi_session_limit_cards_the_pm() {
    let root = init_repo("max-card");
    let launch = vec![
        "sh".to_string(),
        "-c".to_string(),
        "exit 0".to_string(),
        "{prompt}".to_string(),
    ];
    let (decision, asked) = scripted_next(vec![
        Next::Go("GO item fake - ready".to_string()),
        Next::Go("GO item fake - ready".to_string()),
    ]);
    let cards: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let card = |reason: &str| {
        cards.borrow_mut().push(reason.to_string());
        Ok(())
    };
    let now = || {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    };
    let pause = |_seconds: u64| std::thread::sleep(std::time::Duration::from_millis(10));
    let notice = |_note: &str| {};
    let spec = DriveSpec {
        root: &root,
        lab: "deepseek",
        client: "fake-client",
        model: "pinned-model-1",
        launch: &launch,
        prompt: Some("fake session"),
        max_sessions: 2,
        order: None,
        exempt_lease: false,
        poll: 37,
    };
    let hooks = DriveHooks {
        now: &now,
        pause: &pause,
        decision: &decision,
        card: &card,
        notice: &notice,
    };
    let code = run_sessions(&spec, &hooks).expect("two clean sessions then the limit");
    assert_eq!(code, 0);
    assert_eq!(asked.get(), 2, "two sessions were asked for");
    assert_eq!(sessions_started(&root), 2, "two sessions started");
    let cards = cards.into_inner();
    assert_eq!(cards.len(), 1, "the limit cards the PM once: {cards:?}");
    assert!(
        cards[0].starts_with("reached --max-sessions (2 sessions)"),
        "{cards:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

mod gpu_collection_records {
    use std::path::PathBuf;

    use uor_r4_harness::loopdrive::{
        collect_collection, collection_dir, collection_id, read_collections, watch_lease_collect,
        write_collection, GpuCollection,
    };

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("uor-gc-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("temp root");
        root
    }

    fn renew(exit: &str) -> Vec<String> {
        vec!["sh".to_string(), "-c".to_string(), exit.to_string()]
    }

    #[test]
    fn a_lease_watcher_writes_and_updates_a_record_and_collect_marks_it() {
        let root = temp_root("record");
        let (path, notes) = watch_lease_collect(&root, "deepseek", &renew("exit 0"), 1, Some(1))
            .expect("the watcher records a bound-stopped lease");
        assert!(
            path.starts_with(collection_dir(&root, "deepseek")),
            "the record lives under the lab collection dir: {}",
            path.display()
        );
        assert!(
            notes.iter().any(|note| note.contains("test bound")),
            "{notes:?}"
        );
        let records = read_collections(&root, "deepseek").expect("read");
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.status, "stopped");
        assert!(record.ended_unix.is_some());
        assert!(!record.collected);
        assert_eq!(record.lab, "deepseek");

        let (_path, notes) = watch_lease_collect(&root, "deepseek", &renew("exit 1"), 1, Some(1))
            .expect("the watcher records a lease-ended run");
        assert!(
            notes.iter().any(|note| note.contains("lease ended")),
            "{notes:?}"
        );
        let records = read_collections(&root, "deepseek").expect("read again");
        let finished = records
            .iter()
            .find(|record| record.status == "finished")
            .expect("a finished record");
        let artifacts = vec!["bundle/results.sha256".to_string()];
        collect_collection(&root, "deepseek", &finished.id, &artifacts).expect("collect");
        let after = read_collections(&root, "deepseek").expect("read after collect");
        let collected = after
            .iter()
            .find(|record| record.id == finished.id)
            .expect("the record stays");
        assert!(collected.collected);
        assert_eq!(collected.artifacts, artifacts);
        collect_collection(&root, "deepseek", &finished.id, &artifacts)
            .expect("a repeated collect is idempotent");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn malformed_and_foreign_files_are_skipped_and_ids_are_safe() {
        let root = temp_root("skip");
        let directory = collection_dir(&root, "deepseek");
        std::fs::create_dir_all(&directory).expect("collection dir");
        std::fs::write(directory.join("junk.json"), "not json").expect("junk");
        std::fs::write(directory.join("notes.txt"), "x").expect("foreign file");
        assert!(read_collections(&root, "deepseek").unwrap().is_empty());

        let id = collection_id(
            "deep seek",
            &["/usr/local/bin/uor-pod".to_string(), "renew".to_string()],
            42,
        );
        assert_eq!(id, "deep-seek-uorpod-42");
        assert!(!id.contains('/'), "the id is filesystem-safe: {id}");

        let record = GpuCollection {
            id: "deepseek-uorpod-42".to_string(),
            lab: "deepseek".to_string(),
            renew: vec!["uor-pod".to_string()],
            started_unix: 42,
            ended_unix: None,
            status: "running".to_string(),
            collected: false,
            artifacts: Vec::new(),
            notes: Vec::new(),
        };
        write_collection(&root, "deepseek", &record).expect("write");
        assert_eq!(read_collections(&root, "deepseek").unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }
    #[test]
    fn a_watcher_error_leaves_a_stopped_record_not_a_running_one() {
        let root = temp_root("error");
        let missing = vec!["/nonexistent/uor-pod-renew".to_string()];
        assert!(
            watch_lease_collect(&root, "deepseek", &missing, 1, Some(1)).is_err(),
            "a renew program that cannot spawn is an error"
        );
        let records = read_collections(&root, "deepseek").expect("read");
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.status, "stopped");
        assert!(
            record
                .notes
                .iter()
                .any(|note| note.contains("watcher error")),
            "{:?}",
            record.notes
        );
        assert!(record.ended_unix.is_some());
        let _ = std::fs::remove_dir_all(&root);
    }
}
