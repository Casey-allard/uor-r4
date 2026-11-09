#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use uor_r4_desktop_client::{
    DesktopBackend, DesktopClientError, PinRegistry, TerminalEvent, TerminalSnapshot,
};

fn workspace(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "uor-interactive-terminal-{label}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create terminal fixture");
    root
}

fn isolated_backend(root: &Path) -> DesktopBackend {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    backend
        .native_grant_folder(root)
        .expect("grant fixture root");
    let home = root.join("home");
    fs::create_dir_all(&home).expect("create isolated HOME");
    backend
        .native_grant_folder(&home)
        .expect("grant isolated HOME");
    backend.set_terminal_home(&home).expect("set isolated HOME");
    backend.native_set_terminal_enabled(true);
    backend
}

fn output(snapshot: &TerminalSnapshot) -> String {
    snapshot
        .events
        .iter()
        .filter_map(|event| match event {
            TerminalEvent::Output { data_hex, .. } => Some(
                String::from_utf8_lossy(&hex::decode(data_hex).expect("hex output")).into_owned(),
            ),
            _ => None,
        })
        .collect()
}

fn wait_for(
    backend: &mut DesktopBackend,
    session: &str,
    predicate: impl Fn(&TerminalSnapshot) -> bool,
) -> TerminalSnapshot {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = backend.read_terminal(session, 0).expect("read PTY");
        if predicate(&snapshot) {
            return snapshot;
        }
        let rendered = output(&snapshot);
        assert!(
            Instant::now() < deadline,
            "PTY condition was not met before the bounded wait: {rendered:?} {snapshot:?}"
        );
        sleep(Duration::from_millis(25));
    }
}

#[test]
fn backend_is_send_and_sync_for_scoped_host_threads() {
    fn assert_send_and_sync<T: Send + Sync>() {}
    assert_send_and_sync::<DesktopBackend>();
}

#[test]
fn approved_pty_supports_multiple_inputs_resize_and_close() {
    let root = workspace("basic");
    let mut backend = isolated_backend(&root);
    let shell = Path::new("/bin/sh");
    assert!(matches!(
        backend.open_terminal(&root, Some(shell), 24, 80),
        Err(DesktopClientError::TerminalConsentRequired)
    ));
    backend
        .host_approve_terminal_shell(&root, shell)
        .expect("approve fixture shell");
    let opened = backend
        .open_terminal(&root, Some(shell), 24, 80)
        .expect("open PTY");
    let session = opened.session_id.clone();
    backend
        .write_terminal(&session, b"printf 'READY\\n'\n")
        .expect("write first command");
    let ready = wait_for(&mut backend, &session, |snapshot| {
        output(snapshot).contains("READY")
    });
    let serialized = serde_json::to_value(&ready).expect("serialize PTY snapshot");
    let event = serialized["events"]
        .as_array()
        .and_then(|events| events.iter().find(|event| event["type"] == "output"))
        .expect("serialized output event");
    assert!(
        event["dataHex"].is_string(),
        "browser event contract: {event}"
    );
    assert_eq!(event["sessionId"], session.as_str());
    backend
        .write_terminal(&session, b"read answer\n")
        .expect("start read");
    backend
        .write_terminal(&session, b"fixture-answer\n")
        .expect("answer read");
    backend
        .write_terminal(&session, b"printf 'ANSWER:%s\\n' \"$answer\"\n")
        .expect("print answer");
    wait_for(&mut backend, &session, |snapshot| {
        output(snapshot).contains("ANSWER:fixture-answer")
    });
    backend
        .write_terminal(&session, b"pwd\n")
        .expect("request cwd");
    wait_for(&mut backend, &session, |snapshot| {
        output(snapshot).contains(&root.to_string_lossy().to_string())
    });
    backend
        .resize_terminal(&session, 40, 100)
        .expect("resize PTY");
    backend
        .write_terminal(&session, b"stty size\n")
        .expect("request size");
    wait_for(&mut backend, &session, |snapshot| {
        output(snapshot).contains("40 100")
    });
    let closed = backend.close_terminal(&session).expect("close PTY");
    assert!(!closed.active);
    assert!(backend.write_terminal(&session, b"exit\n").is_err());
    assert!(backend.read_terminal(&session, 0).is_err());
    let process = Command::new("ps")
        .args(["-p", &closed.pid.expect("PTY pid").to_string()])
        .output()
        .expect("inspect closed PTY");
    assert!(!process.status.success(), "PTY process survived close");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn pty_shell_approval_is_consumed_per_open() {
    let root = workspace("approval-consume");
    let mut backend = isolated_backend(&root);
    let shell = Path::new("/bin/sh");
    backend
        .host_approve_terminal_shell(&root, shell)
        .expect("approve one fixture shell");
    let first = backend
        .open_terminal(&root, Some(shell), 24, 80)
        .expect("open approved fixture shell");
    backend
        .close_terminal(&first.session_id)
        .expect("close first fixture shell");
    assert!(matches!(
        backend.open_terminal(&root, Some(shell), 24, 80),
        Err(DesktopClientError::TerminalConsentRequired)
    ));
    backend
        .host_approve_terminal_shell(&root, shell)
        .expect("approve a separate fixture shell");
    let second = backend
        .open_terminal(&root, Some(shell), 24, 80)
        .expect("open separately approved fixture shell");
    backend
        .close_terminal(&second.session_id)
        .expect("close second fixture shell");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn pty_enforces_grants_input_output_and_session_bounds() {
    let root = workspace("bounds");
    let mut backend = isolated_backend(&root);
    let shell = Path::new("/bin/sh");
    assert!(backend
        .open_terminal(Path::new("/"), Some(shell), 24, 80)
        .is_err());
    assert!(backend
        .host_approve_terminal_shell(&root, Path::new("/etc/passwd"))
        .is_err());
    backend
        .host_approve_terminal_shell(&root, shell)
        .expect("approve fixture shell");
    let opened = backend
        .open_terminal(&root, Some(shell), 24, 80)
        .expect("open bounded PTY");
    let session = opened.session_id.clone();
    assert!(backend
        .write_terminal(&session, &vec![b'x'; 64 * 1024 + 1])
        .is_err());
    backend
        .write_terminal(&session, b"yes x\n")
        .expect("start output flood");
    wait_for(&mut backend, &session, |snapshot| snapshot.truncated);
    backend
        .close_terminal(&session)
        .expect("close output flood");
    let mut sessions = BTreeSet::new();
    for _ in 0..8 {
        backend
            .host_approve_terminal_shell(&root, shell)
            .expect("approve each intended shell");
        let opened = backend
            .open_terminal(&root, Some(shell), 24, 80)
            .expect("within session cap");
        sessions.insert(opened.session_id.clone());
    }
    assert_eq!(sessions.len(), 8);
    assert!(matches!(
        backend.open_terminal(&root, Some(shell), 24, 80),
        Err(DesktopClientError::SessionLimitReached)
    ));
    for session in sessions {
        backend.close_terminal(&session).expect("close capped PTY");
    }
    let _ = fs::remove_dir_all(&root);
}
