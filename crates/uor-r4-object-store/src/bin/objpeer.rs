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
        die("usage: objpeer info|serve|pull [--store S] [--object F] [--have F] [--bind A] [--addr A] [--out F]");
    }
    let mut store_path = String::from("objpeer.store");
    let mut object = String::new();
    let mut have = String::new();
    let mut bind = String::from("127.0.0.1:0");
    let mut addr = String::new();
    let mut out = String::new();
    let mut sessions: usize = 1;
    let mut concurrent = false;
    let mut expect_sha: Option<[u8; 32]> = None;
    let mut expect_version: Option<[u8; 32]> = None;
    let mut expect_bytes: Option<u64> = None;
    let mut i = 2usize;
    while i < args.len() {
        let key = args[i].clone();
        i += 1;
        if key == "--concurrent" {
            concurrent = true;
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
            "--object" => object = value,
            "--have" => have = value,
            "--bind" => bind = value,
            "--addr" => addr = value,
            "--out" => out = value,
            "--expect-sha256" => expect_sha = Some(hex32(&value)),
            "--expect-version" => expect_version = Some(hex32(&value)),
            "--expect-bytes" => {
                expect_bytes = Some(value.parse().unwrap_or_else(|_| die("bad --expect-bytes")))
            }
            "--sessions" => sessions = value.parse().unwrap_or_else(|_| die("bad --sessions")),
            other => die(&format!("unknown argument {}", other)),
        }
    }
    match args[1].as_str() {
        "info" => {
            if object.is_empty() {
                die("info requires --object");
            }
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
            if object.is_empty() {
                die("serve requires --object");
            }
            let mut store = load_store(&store_path);
            let manifest = fold_file(&mut store, &object);
            save_store(&store, &store_path);
            let listener =
                TcpListener::bind(&bind).unwrap_or_else(|e| die(&format!("bind {}: {}", bind, e)));
            println!(
                "READY {} object_bytes={} roots={} records={} sha256={} manifest_version={}",
                listener.local_addr().unwrap(),
                manifest.byte_len(),
                manifest.roots().len(),
                store.record_count(),
                hex(&manifest.whole_sha256()),
                hex(&manifest.version())
            );
            if concurrent {
                thread::scope(|scope| {
                    for _ in 0..sessions {
                        let (stream, _) = listener
                            .accept()
                            .unwrap_or_else(|e| die(&format!("accept: {}", e)));
                        let store_ref = &store;
                        let manifest_ref = &manifest;
                        scope.spawn(move || {
                            let mut reader = stream
                                .try_clone()
                                .unwrap_or_else(|e| die(&format!("clone: {}", e)));
                            let mut writer = stream;
                            if let Err(error) =
                                peer::serve(store_ref, manifest_ref, &mut reader, &mut writer)
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
                    peer::serve(&store, &manifest, &mut reader, &mut writer)
                        .unwrap_or_else(|e| die(&format!("serve session: {:?}", e)));
                    println!("SERVED");
                }
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
            let (manifest, stats) =
                peer::pull_expected(&mut store, expected, &mut reader, &mut writer)
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
