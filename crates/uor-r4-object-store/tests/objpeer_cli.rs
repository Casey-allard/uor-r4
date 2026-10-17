//! The packaged objpeer CLI replicates a delta between two real processes over loopback.

use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

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
