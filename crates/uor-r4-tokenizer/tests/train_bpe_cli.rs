use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use uor_r4_tokenizer::ByteBpeTokenizer;
static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn cli_trains_real_files_and_refuses_overwrite_and_invalid_utf8() {
    let dir = std::env::temp_dir().join(format!(
        "uor-bpe-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    let input = dir.join("train.txt");
    let output = dir.join("tokenizer.json");
    std::fs::write(&input, "hello hello café λ🦀\n").unwrap();
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_train-bpe"))
            .args(["--input"])
            .arg(&input)
            .arg("--output")
            .arg(&output)
            .args(["--vocab-size", "280", "--min-frequency", "1"])
            .output()
            .unwrap()
    };
    let first = invoke();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let bytes = std::fs::read(&output).unwrap();
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).unwrap();
    let text = "unseen\t日本語🦀";
    assert_eq!(tokenizer.decode(&tokenizer.encode(text)), text);
    assert!(!invoke().status.success());
    assert_eq!(std::fs::read(&output).unwrap(), bytes);
    std::fs::remove_file(&output).unwrap();
    std::fs::write(&input, [255]).unwrap();
    assert!(!invoke().status.success());
    assert!(!output.exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(unix)]
#[test]
fn cli_rejects_fifo_without_waiting_for_a_writer() {
    let dir = std::env::temp_dir().join(format!(
        "uor-bpe-fifo-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    let fifo = dir.join("input.fifo");
    assert!(Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap()
        .success());
    let mut child = Command::new(env!("CARGO_BIN_EXE_train-bpe"))
        .arg("--input")
        .arg(&fifo)
        .arg("--output")
        .arg(dir.join("out.json"))
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let exited = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            break None;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(exited.is_some(), "trainer blocked opening a FIFO");
    assert!(!exited.unwrap().success());
}
