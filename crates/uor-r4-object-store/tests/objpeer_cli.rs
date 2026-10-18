//! The packaged objpeer CLI replicates a delta between two real processes over loopback.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use uor_r4_object_store::{peer, Limits, Store};

fn atoms(n: usize, salt: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n * 32);
    for i in 0..n {
        for j in 0..32 {
            out.push(if j < 8 {
                i.to_le_bytes()[j]
            } else {
                ((i * 37 + j * 11 + salt) & 0xFF) as u8
            });
        }
    }
    out
}

fn tmp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("objpeer-cli-{}-{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[allow(clippy::zombie_processes)]
fn spawn_serve(
    store: &PathBuf,
    object: &PathBuf,
) -> (Child, BufReader<std::process::ChildStdout>, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .args(["serve", "--store"])
        .arg(store)
        .arg("--object")
        .arg(object)
        .args(["--sessions", "1", "--bind", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        assert!(
            reader.read_line(&mut line).unwrap() > 0,
            "serve closed before READY"
        );
        if let Some(addr) = line.strip_prefix("READY ") {
            return (
                child,
                reader,
                addr.split_whitespace().next().unwrap().to_string(),
            );
        }
    }
}

#[test]
fn packaged_cli_replicates_a_delta_between_processes() {
    let dir = tmp_dir("delta");
    let old = dir.join("old.bin");
    let new = dir.join("new.bin");
    let new_bytes = atoms(2064, 1);
    std::fs::write(&old, atoms(2048, 1)).unwrap();
    std::fs::write(&new, &new_bytes).unwrap();

    let (mut server, mut reader, addr) = spawn_serve(&dir.join("server.store"), &new);
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("pull")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--have")
        .arg(&old)
        .arg("--addr")
        .arg(&addr)
        .arg("--out")
        .arg(dir.join("out.bin"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(out.status.success(), "pull failed: {}", stderr);
    assert!(stdout.contains("PULL_STATS"), "missing stats: {}", stdout);
    assert!(
        !stdout.contains("nodes=0 "),
        "expected a delta, got: {}",
        stdout
    );
    assert!(stdout.contains("verified=true"));
    assert_eq!(std::fs::read(dir.join("out.bin")).unwrap(), new_bytes);
    let _ = server.wait();
    let mut rest = String::new();
    reader.read_to_string(&mut rest).ok();
    assert!(
        rest.contains("SERVED"),
        "server should log the completed session: {}",
        rest
    );
}

#[test]
fn packaged_cli_info_folds_an_artifact() {
    let dir = tmp_dir("info");
    let object = dir.join("object.bin");
    std::fs::write(&object, atoms(64, 2)).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .args(["info", "--store"])
        .arg(dir.join("store"))
        .arg("--object")
        .arg(&object)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success());
    assert!(stdout.contains("INFO object_bytes="), "{}", stdout);
    assert!(stdout.contains("records="), "{}", stdout);
}

#[test]
fn packaged_cli_enforces_a_whole_sha256_pin() {
    use sha2::{Digest, Sha256};

    fn sha_hex(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect()
    }

    let dir = tmp_dir("pin");
    let old = dir.join("old.bin");
    let new = dir.join("new.bin");
    let new_bytes = atoms(1040, 4);
    std::fs::write(&old, atoms(1024, 4)).unwrap();
    std::fs::write(&new, &new_bytes).unwrap();
    let correct = sha_hex(&new_bytes);

    let (mut server, mut reader, addr) = spawn_serve(&dir.join("server.store"), &new);
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("pull")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--have")
        .arg(&old)
        .arg("--addr")
        .arg(&addr)
        .arg("--out")
        .arg(dir.join("out.bin"))
        .arg("--expect-sha256")
        .arg(&correct)
        .arg("--expect-bytes")
        .arg(new_bytes.len().to_string())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "pinned pull failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(std::fs::read(dir.join("out.bin")).unwrap(), new_bytes);
    let _ = server.wait();
    let mut rest = String::new();
    reader.read_to_string(&mut rest).ok();

    let (mut server2, mut reader2, addr2) = spawn_serve(&dir.join("server2.store"), &new);
    let out2 = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("pull")
        .arg("--store")
        .arg(dir.join("client2.store"))
        .arg("--have")
        .arg(&old)
        .arg("--addr")
        .arg(&addr2)
        .arg("--out")
        .arg(dir.join("out2.bin"))
        .arg("--expect-sha256")
        .arg("00".repeat(32))
        .arg("--expect-bytes")
        .arg(new_bytes.len().to_string())
        .output()
        .unwrap();
    assert!(!out2.status.success(), "wrong sha pin must fail");
    assert!(
        String::from_utf8_lossy(&out2.stderr).contains("BindingMismatch"),
        "expected a binding failure, got: {}",
        String::from_utf8_lossy(&out2.stderr)
    );
    let _ = server2.wait();
    let mut rest2 = String::new();
    reader2.read_to_string(&mut rest2).ok();
}

#[allow(clippy::zombie_processes)]
fn spawn_serve_many(
    store: &PathBuf,
    objects: &[&PathBuf],
    sessions: usize,
) -> (
    Child,
    BufReader<std::process::ChildStdout>,
    String,
    Vec<String>,
) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_objpeer"));
    command.args(["serve", "--store"]).arg(store);
    for object in objects {
        command.arg("--object").arg(object);
    }
    command
        .args(["--sessions", &sessions.to_string(), "--bind", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        assert!(
            reader.read_line(&mut line).unwrap() > 0,
            "serve closed before READY"
        );
        if let Some(addr) = line.strip_prefix("READY ") {
            let addr = addr.split_whitespace().next().unwrap().to_string();
            let mut object_lines = Vec::new();
            for _ in 0..objects.len() {
                let mut object_line = String::new();
                assert!(reader.read_line(&mut object_line).unwrap() > 0);
                object_lines.push(object_line.trim().to_string());
            }
            return (child, reader, addr, object_lines);
        }
    }
}

#[test]
fn packaged_cli_fleet_verify_and_select() {
    let dir = tmp_dir("fleet");
    let first = dir.join("first.bin");
    let second = dir.join("second.bin");
    std::fs::write(&first, atoms(64, 11)).unwrap();
    std::fs::write(&second, atoms(64, 12)).unwrap();
    let (mut server, mut reader, addr, lines) =
        spawn_serve_many(&dir.join("server.store"), &[&first, &second], 2);
    assert_eq!(lines.len(), 2);
    let sha = lines[0]
        .split("sha256=")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();

    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("verify")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--have")
        .arg(&first)
        .arg("--addr")
        .arg(&addr)
        .arg("--select-sha256")
        .arg(&sha)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "fleet verify failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("verified=true"));

    let out2 = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("verify")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--have")
        .arg(&first)
        .arg("--addr")
        .arg(&addr)
        .arg("--select-sha256")
        .arg("00".repeat(32))
        .output()
        .unwrap();
    assert!(!out2.status.success(), "unknown selector must fail");
    assert!(
        String::from_utf8_lossy(&out2.stderr).contains("UnknownObject"),
        "expected UnknownObject, got: {}",
        String::from_utf8_lossy(&out2.stderr)
    );
    let _ = server.wait();
    let mut rest = String::new();
    reader.read_to_string(&mut rest).ok();
}

#[test]
fn packaged_cli_verify_reports_a_stale_store() {
    let dir = tmp_dir("stale");
    let object = dir.join("object.bin");
    std::fs::write(&object, atoms(64, 21)).unwrap();
    let (mut server, mut reader, addr) = spawn_serve(&dir.join("server.store"), &object);
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("verify")
        .arg("--store")
        .arg(dir.join("empty.store"))
        .arg("--addr")
        .arg(&addr)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(3),
        "stale verify must exit 3: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("verified=false"));
    let _ = server.wait();
    let mut rest = String::new();
    reader.read_to_string(&mut rest).ok();
}

#[test]
fn packaged_cli_batch_verify_uses_one_session() {
    let dir = tmp_dir("pull-batch");
    let first = dir.join("first.bin");
    let second = dir.join("second.bin");
    let third = dir.join("third.bin");
    std::fs::write(&first, atoms(64, 31)).unwrap();
    std::fs::write(&second, atoms(64, 32)).unwrap();
    std::fs::write(&third, atoms(64, 33)).unwrap();
    let (mut server, mut reader, addr, lines) =
        spawn_serve_many(&dir.join("server.store"), &[&first, &second, &third], 1);
    assert_eq!(lines.len(), 3);
    let shas: Vec<String> = lines
        .iter()
        .map(|line| {
            line.split("sha256=")
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    let client = dir.join("client.store");
    for object in [&first, &second] {
        let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
            .args(["info", "--store"])
            .arg(&client)
            .arg("--object")
            .arg(object)
            .output()
            .unwrap();
        assert!(out.status.success());
    }
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("verify")
        .arg("--store")
        .arg(&client)
        .arg("--addr")
        .arg(&addr)
        .args(["--select-sha256", &shas[0]])
        .args(["--select-sha256", &shas[1]])
        .args(["--select-sha256", &shas[2]])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(
        out.status.code(),
        Some(3),
        "batch stale must exit 3: {}",
        stdout
    );
    assert!(stdout.contains("VERIFY i=0") && stdout.contains("verified=true"));
    assert!(stdout.contains("VERIFY i=2") && stdout.contains("verified=false"));
    assert!(stdout.contains("VERIFY_SERIES checks=3 verified=2 stale=1"));
    let _ = server.wait();
    let mut rest = String::new();
    reader.read_to_string(&mut rest).ok();
}

#[test]
fn packaged_cli_verify_targets_file_with_json() {
    let dir = tmp_dir("targets");
    let first = dir.join("first.bin");
    let second = dir.join("second.bin");
    std::fs::write(&first, atoms(64, 51)).unwrap();
    std::fs::write(&second, atoms(64, 52)).unwrap();
    let (mut server, mut reader, addr, lines) =
        spawn_serve_many(&dir.join("server.store"), &[&first, &second], 1);
    let shas: Vec<String> = lines
        .iter()
        .map(|line| {
            line.split("sha256=")
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    let targets = dir.join("targets.txt");
    std::fs::write(
        &targets,
        format!("# fleet targets\n{}\n\n{}\n", shas[0], shas[1]),
    )
    .unwrap();
    let client = dir.join("client.store");
    for object in [&first, &second] {
        let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
            .args(["info", "--store"])
            .arg(&client)
            .arg("--object")
            .arg(object)
            .output()
            .unwrap();
        assert!(out.status.success());
    }
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("verify")
        .arg("--store")
        .arg(&client)
        .arg("--addr")
        .arg(&addr)
        .arg("--targets")
        .arg(&targets)
        .arg("--json")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "targets verify failed: {}", stdout);
    assert!(
        stdout.trim_start().starts_with('{'),
        "expected JSON: {}",
        stdout
    );
    assert!(stdout.contains("\"checks\":2"), "{}", stdout);
    assert!(stdout.contains("\"verified\":2"), "{}", stdout);
    assert!(stdout.contains("\"status\":\"verified\""), "{}", stdout);
    assert!(stdout.contains("\"wire_bytes\":"), "{}", stdout);
    let _ = server.wait();
    let mut rest = String::new();
    reader.read_to_string(&mut rest).ok();
}

#[test]
fn packaged_cli_sync_targets_into_directory() {
    let dir = tmp_dir("sync");
    let first = dir.join("first.bin");
    let second = dir.join("second.bin");
    std::fs::write(&first, atoms(64, 61)).unwrap();
    std::fs::write(&second, atoms(64, 62)).unwrap();
    let (mut server, mut reader, addr, lines) =
        spawn_serve_many(&dir.join("server.store"), &[&first, &second], 1);
    let shas: Vec<String> = lines
        .iter()
        .map(|line| {
            line.split("sha256=")
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    let targets = dir.join("targets.txt");
    std::fs::write(&targets, format!("# sync set\n{}\n{}\n", shas[0], shas[1])).unwrap();
    let out_dir = dir.join("out");
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("sync")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--addr")
        .arg(&addr)
        .arg("--targets")
        .arg(&targets)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--json")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "sync failed: {}", stdout);
    assert!(stdout.contains("\"artifacts\":2"), "{}", stdout);
    for (index, object) in [&first, &second].iter().enumerate() {
        let path = out_dir.join(format!("{}.bin", shas[index]));
        assert_eq!(std::fs::read(path).unwrap(), std::fs::read(object).unwrap());
    }
    let _ = server.wait();
    let mut rest = String::new();
    reader.read_to_string(&mut rest).ok();
}

fn test_limits() -> Limits {
    Limits {
        max_object_bytes: 1 << 30,
        max_records: 1_000_000,
        max_record_bytes: 1 << 30,
        max_returned_bytes: 1 << 30,
        max_visits: 100_000_000,
        max_occurrences: 10_000_000,
    }
}

#[test]
fn packaged_cli_retries_after_a_cut_connection() {
    let dir = tmp_dir("retry");
    let old = dir.join("old.bin");
    let new = dir.join("new.bin");
    let old_bytes = atoms(2048, 71);
    let new_bytes = atoms(2064, 71);
    std::fs::write(&old, &old_bytes).unwrap();
    std::fs::write(&new, &new_bytes).unwrap();

    let mut sender = Store::new(test_limits());
    let manifest = sender.fold(&new_bytes).expect("fold").manifest;
    let server_listener = TcpListener::bind("127.0.0.1:0").expect("bind server");
    let server_addr = server_listener.local_addr().expect("server addr");
    let server = thread::spawn(move || {
        let (stream, _) = server_listener.accept().expect("accept");
        let mut reader = stream.try_clone().expect("clone");
        let mut writer = stream;
        peer::serve(&sender, &manifest, &mut reader, &mut writer).expect("serve");
    });

    let proxy_listener = TcpListener::bind("127.0.0.1:0").expect("bind proxy");
    let proxy_addr = proxy_listener.local_addr().expect("proxy addr");
    let proxy = thread::spawn(move || {
        // First connection: cut after receiving part of the first frame.
        let (mut cut, _) = proxy_listener.accept().expect("accept cut");
        let mut buffer = [0u8; 16];
        let _ = cut.read(&mut buffer);
        drop(cut);
        // Second connection: bridge to the server.
        let (client, _) = proxy_listener.accept().expect("accept bridge");
        let upstream = TcpStream::connect(server_addr).expect("upstream");
        let mut client_read = client.try_clone().expect("clone client");
        let mut upstream_write = upstream.try_clone().expect("clone upstream");
        let pump = thread::spawn(move || {
            let _ = std::io::copy(&mut client_read, &mut upstream_write);
        });
        let mut upstream_read = upstream;
        let mut client_write = client;
        let _ = std::io::copy(&mut upstream_read, &mut client_write);
        pump.join().ok();
    });

    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("pull")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--have")
        .arg(&old)
        .arg("--addr")
        .arg(proxy_addr.to_string())
        .arg("--out")
        .arg(dir.join("out.bin"))
        .args(["--retries", "2"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(out.status.success(), "retry pull failed: {}", stderr);
    assert!(
        stderr.contains("retry 1/2"),
        "expected a retry line in stderr: {}",
        stderr
    );
    assert_eq!(std::fs::read(dir.join("out.bin")).unwrap(), new_bytes);
    server.join().expect("server");
    proxy.join().expect("proxy");
}

#[test]
fn packaged_cli_verify_retries_after_a_cut_connection() {
    use sha2::{Digest, Sha256};

    let dir = tmp_dir("verify-retry");
    let object = dir.join("object.bin");
    let payload = atoms(64, 81);
    std::fs::write(&object, &payload).unwrap();

    let mut sender = Store::new(test_limits());
    let manifest = sender.fold(&payload).expect("fold").manifest;
    let server_listener = TcpListener::bind("127.0.0.1:0").expect("bind server");
    let server_addr = server_listener.local_addr().expect("server addr");
    let server = thread::spawn(move || {
        let (stream, _) = server_listener.accept().expect("accept");
        let mut reader = stream.try_clone().expect("clone");
        let mut writer = stream;
        peer::serve(&sender, &manifest, &mut reader, &mut writer).expect("serve");
    });

    let proxy_listener = TcpListener::bind("127.0.0.1:0").expect("bind proxy");
    let proxy_addr = proxy_listener.local_addr().expect("proxy addr");
    let proxy = thread::spawn(move || {
        let (mut cut, _) = proxy_listener.accept().expect("accept cut");
        let mut buffer = [0u8; 16];
        let _ = cut.read(&mut buffer);
        drop(cut);
        let (client, _) = proxy_listener.accept().expect("accept bridge");
        let upstream = TcpStream::connect(server_addr).expect("upstream");
        let mut client_read = client.try_clone().expect("clone client");
        let mut upstream_write = upstream.try_clone().expect("clone upstream");
        let pump = thread::spawn(move || {
            let _ = std::io::copy(&mut client_read, &mut upstream_write);
        });
        let mut upstream_read = upstream;
        let mut client_write = client;
        let _ = std::io::copy(&mut upstream_read, &mut client_write);
        pump.join().ok();
    });

    let digest = Sha256::digest(&payload);
    let sha: String = digest.iter().map(|byte| format!("{:02x}", byte)).collect();
    let targets = dir.join("targets.txt");
    std::fs::write(&targets, format!("{}\n", sha)).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("verify")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--have")
        .arg(&object)
        .arg("--addr")
        .arg(proxy_addr.to_string())
        .arg("--targets")
        .arg(&targets)
        .args(["--retries", "2"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "verify retry failed: {}", stderr);
    assert!(
        stderr.contains("retry 1/2"),
        "expected retry line: {}",
        stderr
    );
    assert!(
        stdout.contains("verified=1"),
        "expected verified audit: {}",
        stdout
    );
    server.join().expect("server");
    proxy.join().expect("proxy");
}

fn sha_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

#[test]
fn packaged_cli_multi_replica_audit() {
    let dir = tmp_dir("replicas");
    let object = dir.join("object.bin");
    let payload = atoms(64, 91);
    std::fs::write(&object, &payload).unwrap();
    let (mut server1, mut reader1, addr1, _) =
        spawn_serve_many(&dir.join("server1.store"), &[&object], 1);
    let (mut server2, mut reader2, addr2, _) =
        spawn_serve_many(&dir.join("server2.store"), &[&object], 1);
    let client = dir.join("client.store");
    assert!(Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .args(["info", "--store"])
        .arg(&client)
        .arg("--object")
        .arg(&object)
        .output()
        .unwrap()
        .status
        .success());
    let targets = dir.join("targets.txt");
    std::fs::write(&targets, format!("{}\n", sha_hex(&payload))).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("verify")
        .arg("--store")
        .arg(&client)
        .arg("--targets")
        .arg(&targets)
        .arg("--addr")
        .arg(&addr1)
        .arg("--addr")
        .arg(&addr2)
        .arg("--json")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        out.status.success(),
        "multi replica audit failed: {}",
        stdout
    );
    assert!(stdout.contains("\"replicas\":2"), "{}", stdout);
    assert_eq!(
        stdout.matches("\"status\":\"verified\"").count(),
        2,
        "{}",
        stdout
    );
    let _ = server1.wait();
    let _ = server2.wait();
    let mut sink = String::new();
    reader1.read_to_string(&mut sink).ok();
    reader2.read_to_string(&mut sink).ok();
}

#[test]
fn packaged_cli_multi_replica_reports_unknown_on_one_replica() {
    let dir = tmp_dir("replicas-miss");
    let object1 = dir.join("one.bin");
    let object2 = dir.join("two.bin");
    let first = atoms(64, 92);
    let second = atoms(64, 93);
    std::fs::write(&object1, &first).unwrap();
    std::fs::write(&object2, &second).unwrap();
    let (mut server1, mut reader1, addr1, _) =
        spawn_serve_many(&dir.join("server1.store"), &[&object1], 1);
    let (mut server2, mut reader2, addr2, _) =
        spawn_serve_many(&dir.join("server2.store"), &[&object2], 1);
    let client = dir.join("client.store");
    assert!(Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .args(["info", "--store"])
        .arg(&client)
        .arg("--object")
        .arg(&object1)
        .output()
        .unwrap()
        .status
        .success());
    let targets = dir.join("targets.txt");
    std::fs::write(&targets, format!("{}\n", sha_hex(&first))).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("verify")
        .arg("--store")
        .arg(&client)
        .arg("--targets")
        .arg(&targets)
        .arg("--addr")
        .arg(&addr1)
        .arg("--addr")
        .arg(&addr2)
        .arg("--json")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(
        out.status.code(),
        Some(2),
        "expected unknown exit: {}",
        stdout
    );
    assert!(stdout.contains("\"replicas\":2"), "{}", stdout);
    assert_eq!(
        stdout.matches("\"status\":\"verified\"").count(),
        1,
        "{}",
        stdout
    );
    assert_eq!(
        stdout.matches("\"status\":\"unknown\"").count(),
        1,
        "{}",
        stdout
    );
    let _ = server1.wait();
    let _ = server2.wait();
    let mut sink = String::new();
    reader1.read_to_string(&mut sink).ok();
    reader2.read_to_string(&mut sink).ok();
}

#[test]
fn packaged_cli_sync_fails_over_from_a_dead_replica() {
    let dir = tmp_dir("sync-failover-dead");
    let object = dir.join("object.bin");
    let payload = atoms(64, 101);
    std::fs::write(&object, &payload).unwrap();
    let (mut server, mut reader, addr, _) =
        spawn_serve_many(&dir.join("server.store"), &[&object], 1);
    let targets = dir.join("targets.txt");
    std::fs::write(&targets, format!("{}\n", sha_hex(&payload))).unwrap();
    let out_dir = dir.join("out");
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("sync")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--addr")
        .arg("127.0.0.1:1")
        .arg("--addr")
        .arg(&addr)
        .arg("--targets")
        .arg(&targets)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--json")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "failover sync failed: {} {}",
        stdout,
        stderr
    );
    assert!(stderr.contains("unavailable for sync"), "{}", stderr);
    assert!(stdout.contains("\"artifacts\":1"), "{}", stdout);
    assert!(
        stdout.contains(&format!("\"replica\":\"{}\"", addr)),
        "expected second replica in JSON: {}",
        stdout
    );
    assert_eq!(
        std::fs::read(out_dir.join(format!("{}.bin", sha_hex(&payload)))).unwrap(),
        payload
    );
    let _ = server.wait();
    let mut sink = String::new();
    reader.read_to_string(&mut sink).ok();
}

#[test]
fn packaged_cli_sync_fails_over_when_first_replica_lacks_the_target() {
    let dir = tmp_dir("sync-failover-missing");
    let target = dir.join("target.bin");
    let other = dir.join("other.bin");
    let target_payload = atoms(64, 102);
    std::fs::write(&target, &target_payload).unwrap();
    std::fs::write(&other, atoms(64, 103)).unwrap();
    let (mut server1, mut reader1, addr1, _) =
        spawn_serve_many(&dir.join("server1.store"), &[&other], 1);
    let (mut server2, mut reader2, addr2, _) =
        spawn_serve_many(&dir.join("server2.store"), &[&target], 1);
    let targets = dir.join("targets.txt");
    std::fs::write(&targets, format!("{}\n", sha_hex(&target_payload))).unwrap();
    let out_dir = dir.join("out");
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("sync")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--addr")
        .arg(&addr1)
        .arg("--addr")
        .arg(&addr2)
        .arg("--targets")
        .arg(&targets)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--json")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "missing-target failover failed: {} {}",
        stdout,
        stderr
    );
    assert!(stderr.contains("unavailable for sync"), "{}", stderr);
    assert!(
        stdout.contains(&format!("\"replica\":\"{}\"", addr2)),
        "expected second replica in JSON: {}",
        stdout
    );
    let _ = server1.wait();
    let _ = server2.wait();
    let mut sink = String::new();
    reader1.read_to_string(&mut sink).ok();
    reader2.read_to_string(&mut sink).ok();
}

#[test]
fn packaged_cli_repair_restores_split_targets() {
    let dir = tmp_dir("repair");
    let one = dir.join("one.bin");
    let two = dir.join("two.bin");
    let first = atoms(64, 111);
    let second = atoms(64, 112);
    std::fs::write(&one, &first).unwrap();
    std::fs::write(&two, &second).unwrap();
    let (mut server1, mut reader1, addr1, _) =
        spawn_serve_many(&dir.join("server1.store"), &[&one], 3);
    let (mut server2, mut reader2, addr2, _) =
        spawn_serve_many(&dir.join("server2.store"), &[&two], 3);
    let targets = dir.join("targets.txt");
    std::fs::write(
        &targets,
        format!("{}\n{}\n", sha_hex(&first), sha_hex(&second)),
    )
    .unwrap();
    let client = dir.join("client.store");
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("repair")
        .arg("--store")
        .arg(&client)
        .arg("--targets")
        .arg(&targets)
        .arg("--addr")
        .arg(&addr1)
        .arg("--addr")
        .arg(&addr2)
        .arg("--json")
        .arg("--batch")
        .arg("1024")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(out.status.success(), "repair failed: {} {}", stdout, stderr);
    assert!(stdout.contains("\"repaired\":2"), "{}", stdout);
    assert!(stdout.contains("\"unavailable\":0"), "{}", stdout);
    // Prove the local store is now current: pulling from the same replica moves 0 records.
    for (object, address) in [(&one, &addr1), (&two, &addr2)] {
        let sha = sha_hex(&std::fs::read(object).unwrap());
        let pulled = Command::new(env!("CARGO_BIN_EXE_objpeer"))
            .arg("pull")
            .arg("--store")
            .arg(&client)
            .arg("--addr")
            .arg(address)
            .arg("--select-sha256")
            .arg(&sha)
            .arg("--out")
            .arg(dir.join(format!("out-{}.bin", sha)))
            .output()
            .unwrap();
        let pull_stdout = String::from_utf8_lossy(&pulled.stdout).to_string();
        assert!(pulled.status.success(), "{}", pull_stdout);
        assert!(pull_stdout.contains("nodes=0"), "{}", pull_stdout);
    }
    let _ = server1.wait();
    let _ = server2.wait();
    let mut sink = String::new();
    reader1.read_to_string(&mut sink).ok();
    reader2.read_to_string(&mut sink).ok();
}

#[test]
fn packaged_cli_repair_reports_unavailable_targets() {
    let dir = tmp_dir("repair-missing");
    let present = dir.join("present.bin");
    let ghost = atoms(64, 113);
    std::fs::write(&present, atoms(64, 114)).unwrap();
    let (mut server, mut reader, addr, _) =
        spawn_serve_many(&dir.join("server.store"), &[&present], 2);
    let targets = dir.join("targets.txt");
    std::fs::write(
        &targets,
        format!("{}\n{}\n", sha_hex(&atoms(64, 114)), sha_hex(&ghost)),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("repair")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--targets")
        .arg(&targets)
        .arg("--addr")
        .arg(&addr)
        .arg("--json")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(out.status.code(), Some(3), "expected exit 3: {}", stdout);
    assert!(stdout.contains("\"repaired\":1"), "{}", stdout);
    assert!(stdout.contains("\"unavailable\":1"), "{}", stdout);
    let _ = server.wait();
    let mut sink = String::new();
    reader.read_to_string(&mut sink).ok();
}

#[test]
fn packaged_cli_join_pins_a_local_artifact() {
    let dir = tmp_dir("join");
    let object = dir.join("object.bin");
    let payload = atoms(128, 301);
    std::fs::write(&object, &payload).unwrap();
    let sha = sha_hex(&payload);
    let store = dir.join("joined.store");
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("join")
        .arg("--store")
        .arg(&store)
        .arg("--object")
        .arg(&object)
        .arg("--expect-sha256")
        .arg(&sha)
        .arg("--expect-bytes")
        .arg(payload.len().to_string())
        .arg("--json")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        out.status.success(),
        "join failed: {} {}",
        stdout,
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("\"records\":"), "{}", stdout);
    assert!(stdout.contains(&sha), "{}", stdout);
    assert!(store.exists(), "joined store must exist");
    // Idempotent second join.
    let again = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .args(["join", "--store"])
        .arg(&store)
        .arg("--object")
        .arg(&object)
        .arg("--expect-sha256")
        .arg(&sha)
        .output()
        .unwrap();
    assert!(
        again.status.success(),
        "{}",
        String::from_utf8_lossy(&again.stderr)
    );
}

#[test]
fn packaged_cli_join_rejects_a_wrong_pin_without_writing_the_store() {
    let dir = tmp_dir("join-wrong");
    let object = dir.join("object.bin");
    let payload = atoms(64, 302);
    std::fs::write(&object, &payload).unwrap();
    let store = dir.join("wrong.store");
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("join")
        .arg("--store")
        .arg(&store)
        .arg("--object")
        .arg(&object)
        .arg("--expect-sha256")
        .arg("00".repeat(32))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "wrong pin must fail closed");
    assert!(!store.exists(), "wrong pin must not write a store");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("BindingMismatch"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn packaged_cli_pull_batch_flag_pulls_exactly() {
    let dir = tmp_dir("batch");
    let payload = (0..(2 * 1024 * 1024))
        .map(|i| (i % 251) as u8)
        .collect::<Vec<u8>>();
    let object = dir.join("object.bin");
    std::fs::write(&object, &payload).unwrap();
    let sha = sha_hex(&payload);
    let (_server, _reader, addr) = spawn_serve(&dir.join("server.store"), &object);
    let out = dir.join("out.bin");
    let result = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("pull")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--addr")
        .arg(&addr)
        .arg("--select-sha256")
        .arg(&sha)
        .arg("--batch")
        .arg("1024")
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(std::fs::read(&out).unwrap(), payload);
}

#[test]
fn packaged_cli_sync_batch_flag_pulls_exactly() {
    let dir = tmp_dir("sync-batch");
    let payload = (0..(512 * 1024))
        .map(|i| (i % 251) as u8)
        .collect::<Vec<u8>>();
    let object = dir.join("object.bin");
    std::fs::write(&object, &payload).unwrap();
    let sha = sha_hex(&payload);
    let (mut server, mut reader, addr, _lines) =
        spawn_serve_many(&dir.join("server.store"), &[&object], 1);
    let targets = dir.join("targets.txt");
    std::fs::write(&targets, format!("{}\n", sha)).unwrap();
    let out_dir = dir.join("out");
    let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
        .arg("sync")
        .arg("--store")
        .arg(dir.join("client.store"))
        .arg("--addr")
        .arg(&addr)
        .arg("--targets")
        .arg(&targets)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--batch")
        .arg("1024")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "sync --batch failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        std::fs::read(out_dir.join(format!("{}.bin", sha))).unwrap(),
        payload
    );
    let _ = server.wait();
    let mut rest = String::new();
    reader.read_to_string(&mut rest).ok();
}

#[test]
fn packaged_cli_rejects_flags_that_the_subcommand_ignores() {
    let cases: [(&[&str], &str); 5] = [
        (
            &["verify", "--store", "unused", "--batch", "8"],
            "--batch applies only to pull, sync and repair",
        ),
        (
            &["pull", "--store", "unused", "--sessions", "2"],
            "--sessions/--concurrent apply only to serve",
        ),
        (
            &["join", "--store", "unused", "--concurrent"],
            "--sessions/--concurrent apply only to serve",
        ),
        (
            &["verify", "--store", "unused", "--fetch-window", "4"],
            "--fetch-window applies only to pull, sync and repair",
        ),
        (
            &["pull", "--store", "unused", "--fetch-window", "0"],
            "--fetch-window must be 1..=64",
        ),
    ];
    for (args, needle) in cases {
        let out = Command::new(env!("CARGO_BIN_EXE_objpeer"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "args {args:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains(needle), "args {args:?}: {stderr}");
    }
}

fn forward_delayed(mut from: TcpStream, mut to: TcpStream, delay: Duration, bytes: Arc<AtomicU64>) {
    let _ = from.set_read_timeout(Some(Duration::from_secs(10)));
    let mut buf = [0u8; 65536];
    loop {
        match from.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                bytes.fetch_add(n as u64, Ordering::Relaxed);
                if !delay.is_zero() {
                    thread::sleep(delay);
                }
                if to.write_all(&buf[..n]).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    let _ = to.shutdown(Shutdown::Both);
}

/// A small full-duplex TCP relay that sleeps delay per forwarded chunk, standing in for RTT.
fn delayed_proxy(
    upstream: String,
    delay: Duration,
    connections: usize,
) -> (String, Arc<AtomicU64>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind proxy");
    let addr = listener.local_addr().expect("proxy addr").to_string();
    let bytes = Arc::new(AtomicU64::new(0));
    let counter = bytes.clone();
    let handle = thread::spawn(move || {
        for _ in 0..connections {
            let (client, _) = listener.accept().expect("accept proxy");
            let upstream = TcpStream::connect(&upstream).expect("proxy upstream");
            let (client_reader, client_writer) = (client.try_clone().unwrap(), client);
            let (upstream_reader, upstream_writer) = (upstream.try_clone().unwrap(), upstream);
            let a = counter.clone();
            let to_up =
                thread::spawn(move || forward_delayed(client_reader, upstream_writer, delay, a));
            let b = counter.clone();
            let to_client =
                thread::spawn(move || forward_delayed(upstream_reader, client_writer, delay, b));
            let _ = to_up.join();
            let _ = to_client.join();
        }
    });
    (addr, bytes, handle)
}

#[test]
fn packaged_cli_fetch_window_pipelines_under_latency() {
    let dir = tmp_dir("pipeline");
    let payload = atoms(8192, 301);
    let object = dir.join("object.bin");
    std::fs::write(&object, &payload).unwrap();
    let sha = sha_hex(&payload);
    let entries = [(128usize, 1usize), (128, 4), (128, 8), (1024, 4)];
    let (mut server, mut reader, upstream, _) =
        spawn_serve_many(&dir.join("server.store"), &[&object], entries.len());
    let (proxy, bytes, handle) = delayed_proxy(upstream, Duration::from_millis(8), entries.len());
    let mut measured: Vec<(usize, usize, f64)> = Vec::new();
    for (batch, window) in entries {
        let out = dir.join(format!("out-{}-{}.bin", batch, window));
        let client = dir.join(format!("client-{}-{}.store", batch, window));
        let started = Instant::now();
        let result = Command::new(env!("CARGO_BIN_EXE_objpeer"))
            .arg("pull")
            .arg("--store")
            .arg(&client)
            .arg("--addr")
            .arg(&proxy)
            .arg("--select-sha256")
            .arg(&sha)
            .arg("--batch")
            .arg(batch.to_string())
            .arg("--fetch-window")
            .arg(window.to_string())
            .arg("--out")
            .arg(&out)
            .output()
            .unwrap();
        let seconds = started.elapsed().as_secs_f64();
        assert!(
            result.status.success(),
            "pull batch={batch} window={window} failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(std::fs::read(&out).unwrap(), payload);
        println!("PIPELINE batch={batch} window={window} secs={seconds:.3}");
        measured.push((batch, window, seconds));
    }
    handle.join().unwrap();
    let lockstep = measured
        .iter()
        .find(|(batch, window, _)| *batch == 128 && *window == 1)
        .map(|(_, _, seconds)| *seconds)
        .unwrap();
    for (batch, window, seconds) in &measured {
        if *batch == 128 && *window > 1 {
            assert!(
                *seconds < lockstep * 0.75,
                "window {window} ({seconds:.3}s) did not beat lock-step ({lockstep:.3}s) by 25%"
            );
        }
    }
    println!("PIPELINE_PROXY_BYTES={}", bytes.load(Ordering::Relaxed));
    let _ = server.wait();
    let mut sink = String::new();
    reader.read_to_string(&mut sink).ok();
}
