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
