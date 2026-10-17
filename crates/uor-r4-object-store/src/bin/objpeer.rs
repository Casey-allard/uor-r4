//! objpeer: artifact replication over the native UOR-R4 object store.
//!
//! Folds a real file (a container layer, VM/disk image, model artifact or checkpoint) into the
//! content-addressed native store, serves its native manifest over TCP, and pulls only the nodes a
//! peer store is missing. The store persists between runs, so a restarted node stays warm.

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

use uor_r4_object_store::{peer, ExpectedObject, Limits, Manifest, Store};

fn limits() -> Limits {
    Limits {
        max_object_bytes: 1 << 34,
        max_records: 64_000_000,
        max_record_bytes: 1 << 34,
        max_returned_bytes: 1 << 34,
        max_visits: 1_000_000_000,
        max_occurrences: 64_000_000,
    }
}

fn die(msg: &str) -> ! {
    eprintln!("ERROR {}", msg);
    std::process::exit(2);
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex32(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    if value.len() != 64 {
        die("--expect values must be 64 hex characters");
    }
    for index in 0..32 {
        out[index] = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .unwrap_or_else(|_| die("--expect value is not valid hex"));
    }
    out
}

struct Counted<S> {
    inner: S,
    bytes: u64,
}

impl<S: Read> Read for Counted<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.bytes += n as u64;
        Ok(n)
    }
}

impl<S: Write> Write for Counted<S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.bytes += n as u64;
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

fn load_store(path: &str) -> Store {
    let mut store = Store::new(limits());
    if Path::new(path).exists() {
        store
            .load(path)
            .unwrap_or_else(|e| die(&format!("load store {}: {:?}", path, e)));
    }
    store
}

fn save_store(store: &Store, path: &str) {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).ok();
        }
    }
    store
        .save(path)
        .unwrap_or_else(|e| die(&format!("save store {}: {:?}", path, e)));
}

fn fold_file(store: &mut Store, path: &str) -> Manifest {
    let bytes = fs::read(path).unwrap_or_else(|e| die(&format!("read {}: {}", path, e)));
    store
        .fold(&bytes)
        .unwrap_or_else(|e| die(&format!("fold {}: {:?}", path, e)))
        .manifest
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        die("usage: objpeer info|serve|pull|verify [--store S] [--object F ...] [--have F] [--bind A] [--addr A] [--out F] [--select-sha256 HEX] [--expect-sha256 HEX] [--expect-version HEX] [--expect-bytes N] [--sessions N] [--concurrent]");
    }
    let mut store_path = String::from("objpeer.store");
    let mut objects: Vec<String> = Vec::new();
    let mut have = String::new();
    let mut bind = String::from("127.0.0.1:0");
    let mut addr = String::new();
    let mut out = String::new();
    let mut sessions: usize = 1;
    let mut concurrent = false;
    let mut expect_sha: Option<[u8; 32]> = None;
    let mut expect_version: Option<[u8; 32]> = None;
    let mut expect_bytes: Option<u64> = None;
    let mut targets = String::new();
    let mut json = false;
    let mut select_shas: Vec<[u8; 32]> = Vec::new();
    let mut i = 2usize;
    while i < args.len() {
        let key = args[i].clone();
        i += 1;
        if key == "--concurrent" {
            concurrent = true;
            continue;
        }
        if key == "--json" {
            json = true;
            continue;
        }
        let value = if i < args.len() {
            args[i].clone()
        } else {
            die(&format!("missing value for {}", key))
        };
        i += 1;
        match key.as_str() {
            "--store" => store_path = value,
            "--object" => objects.push(value),
            "--have" => have = value,
            "--targets" => targets = value,
            "--bind" => bind = value,
            "--addr" => addr = value,
            "--out" => out = value,
            "--expect-sha256" => expect_sha = Some(hex32(&value)),
            "--expect-version" => expect_version = Some(hex32(&value)),
            "--expect-bytes" => {
                expect_bytes = Some(value.parse().unwrap_or_else(|_| die("bad --expect-bytes")))
            }
            "--select-sha256" => select_shas.push(hex32(&value)),
            "--sessions" => sessions = value.parse().unwrap_or_else(|_| die("bad --sessions")),
            other => die(&format!("unknown argument {}", other)),
        }
    }
    match args[1].as_str() {
        "info" => {
            let object = objects
                .first()
                .cloned()
                .unwrap_or_else(|| die("info requires --object"));
            let mut store = load_store(&store_path);
            let manifest = fold_file(&mut store, &object);
            save_store(&store, &store_path);
            println!(
                "INFO object_bytes={} roots={} records={} sha256={}",
                manifest.byte_len(),
                manifest.roots().len(),
                store.record_count(),
                hex(&manifest.whole_sha256())
            );
        }
        "serve" => {
            if objects.is_empty() {
                die("serve requires at least one --object");
            }
            let mut store = load_store(&store_path);
            let mut manifests = Vec::new();
            for path in &objects {
                manifests.push(fold_file(&mut store, path));
            }
            save_store(&store, &store_path);
            let listener =
                TcpListener::bind(&bind).unwrap_or_else(|e| die(&format!("bind {}: {}", bind, e)));
            println!(
                "READY {} objects={} records={}",
                listener.local_addr().unwrap(),
                manifests.len(),
                store.record_count()
            );
            for manifest in &manifests {
                println!(
                    "OBJECT object_bytes={} roots={} sha256={} manifest_version={}",
                    manifest.byte_len(),
                    manifest.roots().len(),
                    hex(&manifest.whole_sha256()),
                    hex(&manifest.version())
                );
            }
            if concurrent {
                thread::scope(|scope| {
                    for _ in 0..sessions {
                        let (stream, _) = listener
                            .accept()
                            .unwrap_or_else(|e| die(&format!("accept: {}", e)));
                        let store_ref = &store;
                        let manifests_ref = &manifests;
                        scope.spawn(move || {
                            let mut reader = stream
                                .try_clone()
                                .unwrap_or_else(|e| die(&format!("clone: {}", e)));
                            let mut writer = stream;
                            if let Err(error) =
                                peer::serve_many(store_ref, manifests_ref, &mut reader, &mut writer)
                            {
                                eprintln!("session error: {:?}", error);
                            }
                        });
                    }
                });
                println!("SERVED {}", sessions);
            } else {
                for _ in 0..sessions {
                    let (stream, _) = listener
                        .accept()
                        .unwrap_or_else(|e| die(&format!("accept: {}", e)));
                    let mut reader = stream
                        .try_clone()
                        .unwrap_or_else(|e| die(&format!("clone: {}", e)));
                    let mut writer = stream;
                    peer::serve_many(&store, &manifests, &mut reader, &mut writer)
                        .unwrap_or_else(|e| die(&format!("serve session: {:?}", e)));
                    println!("SERVED");
                }
            }
        }
        "verify" => {
            if addr.is_empty() {
                die("verify requires --addr");
            }
            let mut store = load_store(&store_path);
            if !have.is_empty() {
                fold_file(&mut store, &have);
            }
            let stream = TcpStream::connect(&addr)
                .unwrap_or_else(|e| die(&format!("connect {}: {}", addr, e)));
            let reader_stream = stream
                .try_clone()
                .unwrap_or_else(|e| die(&format!("clone: {}", e)));
            let mut reader = Counted {
                inner: reader_stream,
                bytes: 0,
            };
            let mut writer = Counted {
                inner: stream,
                bytes: 0,
            };
            let expected =
                if expect_sha.is_none() && expect_version.is_none() && expect_bytes.is_none() {
                    None
                } else {
                    Some(ExpectedObject {
                        bytes: expect_bytes,
                        version: expect_version,
                        whole_sha256: expect_sha,
                    })
                };
            let mut selectors: Vec<[u8; 32]> = select_shas.clone();
            if !targets.is_empty() {
                let text = fs::read_to_string(&targets)
                    .unwrap_or_else(|e| die(&format!("read {}: {}", targets, e)));
                for (number, line) in text.lines().enumerate() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if line.len() != 64 {
                        die(&format!(
                            "{}:{}: expected 64 hex characters",
                            targets,
                            number + 1
                        ));
                    }
                    selectors.push(hex32(line));
                }
            }
            let mut requests = Vec::new();
            if selectors.is_empty() {
                requests.push(peer::VerifyRequest {
                    expected,
                    selector: None,
                });
            } else {
                for selector in &selectors {
                    requests.push(peer::VerifyRequest {
                        expected,
                        selector: Some(*selector),
                    });
                }
            }
            let outcomes = peer::verify_series(&store, &requests, &mut reader, &mut writer)
                .unwrap_or_else(|e| die(&format!("verify: {:?}", e)));
            let wire_bytes = reader.bytes + writer.bytes;
            let mut verified = 0usize;
            let mut stale = 0usize;
            let mut unknown = 0usize;
            let mut mismatch = 0usize;
            for outcome in &outcomes {
                match outcome {
                    peer::VerifyOutcome::Verified { .. } => verified += 1,
                    peer::VerifyOutcome::Stale { .. } => stale += 1,
                    peer::VerifyOutcome::Unknown => unknown += 1,
                    peer::VerifyOutcome::Mismatch => mismatch += 1,
                }
            }
            if json {
                let results: Vec<String> = outcomes
                    .iter()
                    .enumerate()
                    .map(|(index, outcome)| {
                        let selector = selectors
                            .get(index)
                            .map(|value| hex(value))
                            .unwrap_or_default();
                        match outcome {
                            peer::VerifyOutcome::Verified { missing } => format!(
                                "{{\"i\":{},\"status\":\"verified\",\"missing\":{},\"selector\":\"{}\"}}",
                                index, missing, selector
                            ),
                            peer::VerifyOutcome::Stale { missing } => format!(
                                "{{\"i\":{},\"status\":\"stale\",\"missing\":{},\"selector\":\"{}\"}}",
                                index, missing, selector
                            ),
                            peer::VerifyOutcome::Unknown => format!(
                                "{{\"i\":{},\"status\":\"unknown\",\"selector\":\"{}\"}}",
                                index, selector
                            ),
                            peer::VerifyOutcome::Mismatch => format!(
                                "{{\"i\":{},\"status\":\"mismatch\",\"selector\":\"{}\"}}",
                                index, selector
                            ),
                        }
                    })
                    .collect();
                println!(
                    "{{\"checks\":{},\"verified\":{},\"stale\":{},\"unknown\":{},\"mismatch\":{},\"wire_bytes\":{},\"results\":[{}]}}",
                    outcomes.len(),
                    verified,
                    stale,
                    unknown,
                    mismatch,
                    wire_bytes,
                    results.join(",")
                );
            } else {
                for (index, outcome) in outcomes.iter().enumerate() {
                    match outcome {
                        peer::VerifyOutcome::Verified { missing } => {
                            println!("VERIFY i={} missing={} verified=true", index, missing);
                        }
                        peer::VerifyOutcome::Stale { missing } => {
                            println!("VERIFY i={} missing={} verified=false", index, missing);
                        }
                        peer::VerifyOutcome::Unknown => {
                            eprintln!("VERIFY i={} UnknownObject", index);
                        }
                        peer::VerifyOutcome::Mismatch => {
                            eprintln!("VERIFY i={} BindingMismatch", index);
                        }
                    }
                }
                println!(
                    "VERIFY_SERIES checks={} verified={} stale={} unknown={} mismatch={} wire_bytes={}",
                    outcomes.len(),
                    verified,
                    stale,
                    unknown,
                    mismatch,
                    wire_bytes
                );
            }
            if unknown > 0 {
                std::process::exit(2);
            }
            if stale > 0 || mismatch > 0 {
                std::process::exit(3);
            }
        }
        "pull" => {
            if addr.is_empty() || out.is_empty() {
                die("pull requires --addr and --out");
            }
            let mut store = load_store(&store_path);
            if !have.is_empty() {
                fold_file(&mut store, &have);
            }
            save_store(&store, &store_path);
            let stream = TcpStream::connect(&addr)
                .unwrap_or_else(|e| die(&format!("connect {}: {}", addr, e)));
            let reader_stream = stream
                .try_clone()
                .unwrap_or_else(|e| die(&format!("clone: {}", e)));
            let mut reader = Counted {
                inner: reader_stream,
                bytes: 0,
            };
            let mut writer = Counted {
                inner: stream,
                bytes: 0,
            };
            let expected =
                if expect_sha.is_none() && expect_version.is_none() && expect_bytes.is_none() {
                    None
                } else {
                    Some(ExpectedObject {
                        bytes: expect_bytes,
                        version: expect_version,
                        whole_sha256: expect_sha,
                    })
                };
            let (manifest, stats) = peer::pull_selected(
                &mut store,
                expected,
                select_shas.first().copied(),
                &mut reader,
                &mut writer,
            )
            .unwrap_or_else(|e| die(&format!("pull: {:?}", e)));
            let wire_bytes = reader.bytes + writer.bytes;
            save_store(&store, &store_path);
            let verified = store
                .rebuild(&manifest)
                .unwrap_or_else(|e| die(&format!("rebuild: {:?}", e)));
            fs::write(&out, verified.bytes())
                .unwrap_or_else(|e| die(&format!("write {}: {}", out, e)));
            println!(
                "PULL_STATS object_bytes={} nodes={} record_bytes={} wire_bytes={} rounds={} verified={} sha256={}",
                manifest.byte_len(),
                stats.nodes_requested,
                stats.record_bytes,
                wire_bytes,
                stats.rounds,
                stats.verified,
                hex(&manifest.whole_sha256())
            );
        }
        other => die(&format!("unknown command {}", other)),
    }
}
